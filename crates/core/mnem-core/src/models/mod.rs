pub use semantic_delta_protocol::models::{
    Chunk, RecordKind, SemanticRecord, SemanticSymbol, SymbolReference,
};

/// A [`SemanticSymbol`] enriched with additional structural information
/// extracted from the Tree-sitter AST by [`crate::storage::semantic_enrichment::SemanticEnricher`].
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EnrichedSemanticSymbol {
    /// The base symbol metadata (name, kind, scope, hashes, byte offsets, …).
    #[serde(flatten)]
    pub symbol: SemanticSymbol,
    /// The function / definition signature without the body block.
    pub signature: Option<String>,
    /// Leading doc-comment stripped of comment markers.
    pub docstring: Option<String>,
    /// McCabe cyclomatic complexity (1 + number of decision-point nodes).
    pub cyclomatic_complexity: usize,
}
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Snapshot {
    pub id: i64,
    pub file_path: String,
    pub timestamp: String,
    pub content_hash: String, // This could now be the "root" hash or just a reference
    pub git_branch: Option<String>,
    pub session_id: Option<i64>,
    pub commit_hash: Option<String>,
    pub commit_message: Option<String>,
    pub checkpoint_name: Option<String>,
}

pub struct FileEntry {
    pub path: String,
    pub last_update: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SearchResult {
    pub file_path: String,
    pub content_hash: String,
    pub timestamp: String,
    pub git_branch: Option<String>,
    pub line_number: usize,
    pub content: String,
}

#[derive(Clone, Debug)]
pub struct Session {
    pub id: i64,
    pub start_time: String,
    pub end_time: Option<String>,
    pub git_branch: Option<String>,
    pub file_count: usize,
    pub snapshot_count: usize,
}

#[derive(Clone, Debug)]
pub struct TimesheetEntry {
    pub date: String,
    pub branch: Option<String>,
    pub duration_minutes: u64,
    pub file_count: usize,
    pub snapshot_count: usize,
}

#[derive(Debug, Clone)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: BTreeMap<String, FileNode>,
    pub is_open: bool,
}

impl FileNode {
    pub fn new(name: String, path: String, is_dir: bool) -> Self {
        Self {
            name,
            path,
            is_dir,
            children: BTreeMap::new(),
            is_open: true,
        }
    }

    pub fn insert_path(&mut self, full_path: &str) {
        let parts: Vec<&str> = full_path.split('/').filter(|s| !s.is_empty()).collect();
        self.insert_recursive(&parts, "");
    }

    fn insert_recursive(&mut self, parts: &[&str], current_path: &str) {
        let Some(name) = parts.first() else {
            return;
        };

        let new_path = if current_path.is_empty() {
            name.to_string()
        } else {
            format!("{}/{}", current_path, name)
        };

        let is_dir = parts.len() > 1;

        let entry = self
            .children
            .entry(name.to_string())
            .or_insert_with(|| FileNode::new(name.to_string(), new_path.clone(), is_dir));

        if is_dir {
            entry.is_dir = true;
            entry.insert_recursive(&parts[1..], &new_path);
        }
    }

    pub fn flatten(&self, indent: usize, out: &mut Vec<(FileNode, usize)>) {
        for node in self.children.values() {
            out.push((node.clone(), indent));
            if node.is_dir && node.is_open {
                node.flatten(indent + 1, out);
            }
        }
    }
}

pub mod project;

pub use project::Project;
