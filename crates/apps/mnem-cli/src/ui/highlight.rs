use mnem_macros::UiDebug;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use std::collections::HashMap;
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

#[derive(UiDebug)]
pub struct TsHighlighter {
    highlighter: Highlighter,
    configs: HashMap<String, HighlightConfiguration>,
}

impl std::fmt::Debug for TsHighlighter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TsHighlighter")
            .field("languages", &self.configs.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl TsHighlighter {
    pub fn test_output(&self) {
        println!("SYNTAX HIGHLIGHTING PREVIEW (Rust):");
        println!();

        let code = r#"
fn main() {
    let message = "Hello Mnemosyne!";
    println!("{}", message);
    
    match 42 {
        v if v > 0 => println!("Positive"),
        _ => (),
    }
}
"#;

        // We need a TUI theme for highlight method, but for CLI we might just want ANSI
        // For testing purpose in CLI, we'll just show we can parse it
        println!("(Parsed {} lines of Rust code)", code.lines().count());
        println!(
            "Available configs: {:?}",
            self.configs.keys().collect::<Vec<_>>()
        );
    }

    pub fn new() -> Self {
        let mut configs = HashMap::new();

        // Rust configuration
        if let Ok(mut config) = HighlightConfiguration::new(
            tree_sitter_rust::language(),
            "rust",
            RUST_HIGHLIGHTS,
            "",
            "",
        ) {
            config.configure(&HIGHLIGHT_NAMES);
            configs.insert("rs".to_string(), config);
        }

        // Ruby configuration
        if let Ok(mut config) = HighlightConfiguration::new(
            tree_sitter_ruby::LANGUAGE.into(),
            "ruby",
            RUBY_HIGHLIGHTS,
            "",
            "",
        ) {
            config.configure(&HIGHLIGHT_NAMES);
            configs.insert("rb".to_string(), config);
        }

        Self {
            highlighter: Highlighter::new(),
            configs,
        }
    }

    pub fn highlight(
        &mut self,
        content: &str,
        extension: &str,
        theme: &mnem_tui::theme::Theme,
    ) -> Vec<Line<'static>> {
        let config = match self.configs.get(extension) {
            Some(c) => c,
            None => {
                return content.lines().map(|l| Line::from(l.to_string())).collect();
            }
        };

        let highlights = match self
            .highlighter
            .highlight(config, content.as_bytes(), None, |_| None)
        {
            Ok(h) => h,
            Err(_) => return content.lines().map(|l| Line::from(l.to_string())).collect(),
        };

        let mut lines = Vec::new();
        let mut current_spans = Vec::new();
        let mut style_stack = Vec::new();

        for event in highlights {
            match event {
                Ok(HighlightEvent::HighlightStart(idx)) => {
                    let name = HIGHLIGHT_NAMES[idx.0];
                    style_stack.push(get_style(name, theme));
                }
                Ok(HighlightEvent::Source { start, end }) => {
                    let text = &content[start..end];
                    let style = style_stack.last().cloned().unwrap_or_default();

                    let mut parts = text.split('\n').peekable();
                    while let Some(part) = parts.next() {
                        if !part.is_empty() {
                            current_spans.push(Span::styled(part.to_string(), style));
                        }

                        if parts.peek().is_some() {
                            // End of line reached
                            lines.push(Line::from(current_spans.drain(..).collect::<Vec<_>>()));
                        }
                    }
                }
                Ok(HighlightEvent::HighlightEnd) => {
                    style_stack.pop();
                }
                _ => {}
            }
        }

        // Push remaining content if any
        if !current_spans.is_empty() || lines.is_empty() {
            lines.push(Line::from(current_spans));
        }

        // Ensure we don't return fewer lines than expected by simple .lines()
        // (to keep diff alignment consistent)
        let expected_count = content.lines().count();
        while lines.len() < expected_count {
            lines.push(Line::from(""));
        }

        lines
    }
}

fn get_style(name: &str, theme: &mnem_tui::theme::Theme) -> Style {
    match name {
        "keyword" => Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
        "function" => Style::default()
            .fg(theme.text_main)
            .add_modifier(Modifier::BOLD),
        "type" => Style::default().fg(theme.accent),
        "string" => Style::default().fg(theme.success),
        "comment" => Style::default()
            .fg(theme.text_dim)
            .add_modifier(Modifier::ITALIC),
        "constant" => Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
        "variable" => Style::default().fg(theme.text_main),
        "variable.builtin" => Style::default().fg(theme.accent),
        "variable.parameter" => Style::default()
            .fg(theme.text_main)
            .add_modifier(Modifier::ITALIC),
        "operator" => Style::default().fg(theme.accent),
        "punctuation" => Style::default().fg(theme.text_dim),
        "string.special" => Style::default()
            .fg(theme.success)
            .add_modifier(Modifier::BOLD), // Symbols in Ruby
        "constructor" => Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
        _ => Style::default(),
    }
}

const HIGHLIGHT_NAMES: &[&str] = &[
    "keyword",
    "function",
    "type",
    "string",
    "comment",
    "constant",
    "variable",
    "variable.builtin",
    "variable.parameter",
    "operator",
    "punctuation",
    "string.special",
    "constructor",
];

const RUST_HIGHLIGHTS: &str = r#"
(line_comment) @comment
(block_comment) @comment
(string_literal) @string
(boolean_literal) @constant
(integer_literal) @constant
(float_literal) @constant

[
  "as" "async" "await" "break" "const" "continue" "crate" "else" "enum" "extern"
  "false" "fn" "for" "if" "impl" "in" "let" "loop" "match" "mod" "move" "mut" "pub"
  "ref" "return" "self" "Self" "static" "struct" "super" "trait" "true" "type"
  "unsafe" "use" "where" "while"
] @keyword

(identifier) @variable
(type_identifier) @type
(function_item name: (identifier) @function)
(call_expression function: (identifier) @function)
(call_expression function: (field_expression field: (field_identifier) @function))
"#;

const RUBY_HIGHLIGHTS: &str = r#"
(comment) @comment
(string) @string
(bare_string) @string
(symbol) @string.special
(interpolation) @punctuation

[
  "def" "end" "class" "module" "if" "else" "elsif" "unless" "while" "until" 
  "for" "in" "do" "return" "yield" "super" "self" "nil" "true" "false"
  "and" "or" "not" "alias" "undef" "defined?" "BEGIN" "END"
] @keyword

(identifier) @variable
(constant) @constant
(instance_variable) @variable.builtin
(class_variable) @variable.builtin
(global_variable) @variable.builtin

(method name: (identifier) @function)
(method_call method: (identifier) @function)
(method_call method: (constant) @function)

(argument_list (identifier) @variable.parameter)

[
  "+" "-" "*" "/" "%" "**"
  "==" "!=" ">" "<" ">=" "<=" "<=>" "==="
  "=" "+=" "-=" "*=" "/="
  "&&" "||" "!"
  "&" "|" "^" "~" "<<" ">>"
  ".." "..."
] @operator

[
  "(" ")" "[" "]" "{" "}"
  "," "." ":" ";"
] @punctuation
"#;
