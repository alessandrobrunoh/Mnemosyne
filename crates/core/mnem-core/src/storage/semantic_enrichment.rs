//! Semantic enrichment for Tree-sitter parsed symbols.
//!
//! This module provides [`SemanticEnricher`] which, given raw file content and
//! the list of [`crate::models::SemanticSymbol`] byte-ranges produced by the
//! `SemanticParser`, re-parses the source with Tree-sitter and extracts:
//!
//! * **Signature** – the "header" portion of a definition (parameters, return
//!   type) without the body block, making retrieval summaries useful without
//!   loading the full source.
//! * **Docstring** – the leading doc-comment immediately preceding the node,
//!   stripped of comment markers.
//! * **Cyclomatic complexity** – an approximation of McCabe complexity:
//!   `1 + <number of branching nodes>`.  This replaces the previous trivial
//!   line-count heuristic.

use bytes::Bytes;
use tree_sitter::{Language, Node, Parser};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Enrichment data extracted from the Tree-sitter AST for a single symbol.
#[derive(Debug, Clone, Default)]
pub struct SymbolEnrichment {
    /// The textual signature of the definition, e.g. `fn foo(x: i32) -> bool`.
    /// `None` when the node has no distinct header / body split (e.g. a plain
    /// variable binding).
    pub signature: Option<String>,
    /// Doc comment text immediately preceding the symbol node, with comment
    /// markers stripped.  `None` when no leading comment is present.
    pub docstring: Option<String>,
    /// McCabe cyclomatic complexity: `1 + <number of decision-point nodes>`.
    /// Minimum value is `1`.
    pub cyclomatic_complexity: usize,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Enriches a list of symbol byte-ranges with structural information extracted
/// from a fresh Tree-sitter parse of the source file.
pub struct SemanticEnricher;

impl SemanticEnricher {
    /// Given file `content` and its `extension`, compute [`SymbolEnrichment`]
    /// for every `(start_byte, end_byte)` pair in `symbol_ranges`.
    ///
    /// The returned `Vec` is aligned with `symbol_ranges`: index `i` in the
    /// result corresponds to index `i` in `symbol_ranges`.
    ///
    /// If the language is unsupported or parsing fails, each entry will be a
    /// default enrichment with `cyclomatic_complexity == 1`.
    pub fn enrich(
        content: &Bytes,
        extension: &str,
        symbol_ranges: &[(usize, usize)],
    ) -> Vec<SymbolEnrichment> {
        let default_enrichments = || {
            symbol_ranges
                .iter()
                .map(|_| SymbolEnrichment {
                    cyclomatic_complexity: 1,
                    ..Default::default()
                })
                .collect::<Vec<_>>()
        };

        let lang = match get_language(extension) {
            Some(l) => l,
            None => return default_enrichments(),
        };

        let mut parser = Parser::new();
        if parser.set_language(&lang).is_err() {
            return default_enrichments();
        }

        let tree = match parser.parse(content.as_ref(), None) {
            Some(t) => t,
            None => return default_enrichments(),
        };

        let root = tree.root_node();
        let source = content.as_ref();

        symbol_ranges
            .iter()
            .map(|&(start, end)| {
                match find_node_at(root, start, end) {
                    Some(node) => SymbolEnrichment {
                        signature: extract_signature(node, source, extension),
                        docstring: extract_docstring(node, source),
                        cyclomatic_complexity: cyclomatic_complexity(node).max(1),
                    },
                    None => SymbolEnrichment {
                        cyclomatic_complexity: 1,
                        ..Default::default()
                    },
                }
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Language registry (mirrors semantic-delta-protocol's registry)
// ---------------------------------------------------------------------------

fn get_language(extension: &str) -> Option<Language> {
    match extension {
        "rs" => Some(tree_sitter_rust::language().into()),
        "py" => Some(tree_sitter_python::LANGUAGE.into()),
        "js" | "jsx" => Some(tree_sitter_javascript::LANGUAGE.into()),
        "ts" => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
        "tsx" => Some(tree_sitter_typescript::LANGUAGE_TSX.into()),
        "go" => Some(tree_sitter_go::LANGUAGE.into()),
        "c" | "h" => Some(tree_sitter_c::LANGUAGE.into()),
        "cpp" | "hpp" | "cc" | "cxx" => Some(tree_sitter_cpp::LANGUAGE.into()),
        "java" => Some(tree_sitter_java::LANGUAGE.into()),
        "rb" => Some(tree_sitter_ruby::LANGUAGE.into()),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Node lookup
// ---------------------------------------------------------------------------

/// Descend from `root` to the most-specific node whose byte range exactly
/// covers `[start, end)`.  Falls back to any ancestor that fully contains the
/// range if no exact match exists.
fn find_node_at<'a>(root: Node<'a>, start: usize, end: usize) -> Option<Node<'a>> {
    if root.start_byte() > start || root.end_byte() < end {
        return None;
    }
    let mut node = root;
    'outer: loop {
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                if child.start_byte() <= start && child.end_byte() >= end {
                    node = child;
                    continue 'outer;
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        break;
    }
    if node.start_byte() <= start && node.end_byte() >= end {
        Some(node)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Signature extraction
// ---------------------------------------------------------------------------

/// Body-block node kinds for each language family.
const BODY_KINDS_BRACED: &[&str] = &["block", "statement_block", "compound_statement"];
const BODY_KINDS_PYTHON: &[&str] = &["block"];
const BODY_KINDS_RUBY: &[&str] = &["body_statement", "do_block", "then"];

fn body_kinds_for(extension: &str) -> &'static [&'static str] {
    match extension {
        "py" => BODY_KINDS_PYTHON,
        "rb" => BODY_KINDS_RUBY,
        _ => BODY_KINDS_BRACED,
    }
}

/// Extract a human-readable signature for a node by capturing everything from
/// the start of the node up to (but not including) the body block.
fn extract_signature(node: Node, source: &[u8], extension: &str) -> Option<String> {
    let body_kinds = body_kinds_for(extension);
    let mut body_start: Option<usize> = None;

    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            let child = cursor.node();
            if body_kinds.contains(&child.kind()) {
                body_start = Some(child.start_byte());
                break;
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }

    let raw = match body_start {
        Some(end) => source.get(node.start_byte()..end)?,
        // No body found – use the full node text (e.g. struct fields, type aliases)
        None => return node.utf8_text(source).ok().map(|s| s.trim().to_string()),
    };

    let text = std::str::from_utf8(raw).ok()?.trim().to_string();
    if text.is_empty() { None } else { Some(text) }
}

// ---------------------------------------------------------------------------
// Docstring extraction
// ---------------------------------------------------------------------------

/// Node kinds that are considered documentation / comments.
const COMMENT_KINDS: &[&str] = &[
    "line_comment",
    "block_comment",
    "comment",
    "doc_comment",
    "documentation",
];

/// Find the comment node immediately preceding `node` among its siblings and
/// return its cleaned text.  Only a contiguous run of comment nodes is
/// considered; any non-comment between the candidate and the target resets the
/// search.
fn extract_docstring(node: Node, source: &[u8]) -> Option<String> {
    let parent = node.parent()?;
    let mut cursor = parent.walk();
    if !cursor.goto_first_child() {
        return None;
    }

    let mut prev_comment: Option<String> = None;
    loop {
        let child = cursor.node();
        if child == node {
            return prev_comment;
        }
        if COMMENT_KINDS.contains(&child.kind()) {
            if let Ok(text) = child.utf8_text(source) {
                prev_comment = Some(clean_comment(text));
            }
        } else {
            // Reset – there is a non-comment node between the comment and the symbol.
            prev_comment = None;
        }
        if !cursor.goto_next_sibling() {
            break;
        }
    }
    None
}

/// Strip comment markers and leading/trailing whitespace from comment text.
fn clean_comment(raw: &str) -> String {
    let mut lines: Vec<&str> = Vec::new();
    for line in raw.lines() {
        let t = line.trim();
        let cleaned = if t.starts_with("///") {
            t.trim_start_matches("///").trim()
        } else if t.starts_with("//!") {
            t.trim_start_matches("//!").trim()
        } else if t.starts_with("//") {
            t.trim_start_matches("//").trim()
        } else if t.starts_with("/**") {
            t.trim_start_matches("/**").trim_end_matches("*/").trim()
        } else if t.starts_with("/*") {
            t.trim_start_matches("/*").trim_end_matches("*/").trim()
        } else if t.starts_with('*') {
            t.trim_start_matches('*').trim()
        } else if t.starts_with('#') {
            t.trim_start_matches('#').trim()
        } else {
            t
        };
        if !cleaned.is_empty() {
            lines.push(cleaned);
        }
    }
    lines.join("\n")
}

// ---------------------------------------------------------------------------
// Cyclomatic complexity
// ---------------------------------------------------------------------------

/// Decision-point node kinds across languages.
///
/// Each encountered node of one of these kinds contributes +1 to the count.
const BRANCH_KINDS: &[&str] = &[
    // Conditionals
    "if_expression",
    "if_statement",
    "elif_clause",
    "else_clause",
    "conditional_expression",
    "ternary_expression",
    // Loops
    "for_expression",
    "for_statement",
    "for_in_statement",
    "while_expression",
    "while_statement",
    "loop_expression",
    "do_statement",
    // Pattern matching / switch
    "match_arm",
    "case_clause",
    "switch_case",
    "switch_label",
    // Exception handling
    "catch_clause",
    "except_clause",
    "rescue_clause",
    // Short-circuit operators (anonymous nodes in most grammars)
    "&&",
    "||",
    "and",
    "or",
    // Go-specific
    "select_statement",
    "comm_clause",
];

/// Compute McCabe cyclomatic complexity: `1 + <number of branch nodes>`.
///
/// Uses the Tree-sitter recommended iterative DFS pattern to avoid potential
/// stack-overflow on very deep ASTs and to correctly handle the cursor API.
fn cyclomatic_complexity(node: Node) -> usize {
    let mut count = 1usize;
    let mut cursor = node.walk();
    let mut reached_end = false;

    while !reached_end {
        let current = cursor.node();
        if BRANCH_KINDS.contains(&current.kind()) {
            count += 1;
        }

        if cursor.goto_first_child() {
            continue;
        }
        if cursor.goto_next_sibling() {
            continue;
        }
        // Backtrack: go up until we find a next sibling or exhaust the subtree.
        loop {
            if !cursor.goto_parent() {
                reached_end = true;
                break;
            }
            // Stop when we return to the original node's level.
            if cursor.node() == node {
                reached_end = true;
                break;
            }
            if cursor.goto_next_sibling() {
                break;
            }
        }
    }

    count
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    // ------------------------------------------------------------------
    // Helpers
    // ------------------------------------------------------------------

    fn enrich_first(src: &str, ext: &str) -> SymbolEnrichment {
        // Use full-file range so we always match the root node.
        let content = Bytes::from(src.to_string());
        let ranges = vec![(0, src.len())];
        let mut result = SemanticEnricher::enrich(&content, ext, &ranges);
        result.pop().unwrap()
    }

    // ------------------------------------------------------------------
    // Cyclomatic complexity
    // ------------------------------------------------------------------

    #[test]
    fn complexity_simple_fn_is_one() {
        let src = "fn hello() { println!(\"hi\"); }";
        let e = enrich_first(src, "rs");
        assert_eq!(e.cyclomatic_complexity, 1);
    }

    #[test]
    fn complexity_counts_if() {
        // one `if` => complexity 2
        let src = "fn check(x: i32) -> bool { if x > 0 { true } else { false } }";
        let e = enrich_first(src, "rs");
        assert!(
            e.cyclomatic_complexity >= 2,
            "expected ≥ 2, got {}",
            e.cyclomatic_complexity
        );
    }

    #[test]
    fn complexity_counts_multiple_branches() {
        // No leading newline so (0, src.len()) maps exactly to the function_item node.
        let src = concat!(
            "fn classify(x: i32) -> &'static str {",
            " if x < 0 { \"negative\" } else if x == 0 { \"zero\" } else { \"positive\" } }"
        );
        let e = enrich_first(src, "rs");
        assert!(
            e.cyclomatic_complexity >= 3,
            "expected ≥ 3, got {}",
            e.cyclomatic_complexity
        );
    }

    #[test]
    fn complexity_minimum_is_one() {
        // Even for empty / trivial input, minimum must be 1.
        let e = enrich_first("", "rs");
        assert!(e.cyclomatic_complexity >= 1);
    }

    // ------------------------------------------------------------------
    // Signature extraction
    // ------------------------------------------------------------------

    #[test]
    fn signature_captures_function_header() {
        let src = "fn add(a: i32, b: i32) -> i32 { a + b }";
        let e = enrich_first(src, "rs");
        let sig = e.signature.expect("signature should be present");
        assert!(sig.contains("fn add"), "sig: {}", sig);
        assert!(sig.contains("a: i32"), "sig: {}", sig);
        assert!(!sig.contains("a + b"), "body leaked into signature: {}", sig);
    }

    // ------------------------------------------------------------------
    // Docstring extraction
    // ------------------------------------------------------------------

    #[test]
    fn docstring_extracted_from_line_comment() {
        // No leading/trailing newlines so the byte ranges match precisely.
        let src = "/// Adds two numbers together.\nfn add(a: i32, b: i32) -> i32 { a + b }";
        // Use the function's byte range rather than whole file.
        let content = Bytes::from(src.to_string());
        // Find where `fn add` starts; that is the function_item's start_byte.
        let fn_start = src.find("fn add").expect("fn add not found");
        // The function ends at the last `}`.
        let fn_end = src.len();
        let ranges = vec![(fn_start, fn_end)];
        let enrichments = SemanticEnricher::enrich(&content, "rs", &ranges);
        let e = &enrichments[0];
        assert!(
            e.docstring.is_some(),
            "Expected docstring to be present"
        );
        let doc = e.docstring.as_ref().unwrap();
        assert!(
            doc.contains("Adds two numbers"),
            "Unexpected docstring: {}",
            doc
        );
    }

    // ------------------------------------------------------------------
    // Unsupported language
    // ------------------------------------------------------------------

    #[test]
    fn unsupported_language_returns_defaults() {
        let src = "x = 1 + 2;";
        let content = Bytes::from(src.to_string());
        let ranges = vec![(0, src.len())];
        let enrichments = SemanticEnricher::enrich(&content, "unknown_ext", &ranges);
        assert_eq!(enrichments.len(), 1);
        assert_eq!(enrichments[0].cyclomatic_complexity, 1);
        assert!(enrichments[0].signature.is_none());
        assert!(enrichments[0].docstring.is_none());
    }

    // ------------------------------------------------------------------
    // clean_comment helper
    // ------------------------------------------------------------------

    #[test]
    fn clean_comment_strips_slashes() {
        assert_eq!(clean_comment("/// Hello world"), "Hello world");
        assert_eq!(clean_comment("// Comment"), "Comment");
    }

    #[test]
    fn clean_comment_strips_block_markers() {
        assert_eq!(clean_comment("/** Hello */"), "Hello");
        assert_eq!(clean_comment("/* Hello */"), "Hello");
    }
}
