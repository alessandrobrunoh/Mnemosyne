use anyhow::Result;
use async_trait::async_trait;
use mnem_core::{
    client::DaemonClient,
    protocol::{methods, SymbolHistoryEntry},
};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::{lsp_types::*, Client, LanguageServer, LspService, Server};

const CMD_VIEW_HISTORY: &str = "mnemosyne.viewHistory";
const CMD_VIEW_DIFF: &str = "mnemosyne.viewDiff";
const CMD_RESTORE_SYMBOL: &str = "mnemosyne.restoreSymbol";
const CMD_COMPARE_WITH_VERSION: &str = "mnemosyne.compareWithVersion";

struct Backend {
    client: Client,
    mnem_client: Arc<Mutex<Option<DaemonClient>>>,
    documents: Arc<Mutex<HashMap<Url, String>>>,
    daemon_fail_count: Arc<Mutex<u32>>,
    /// Cached path to the Zed CLI binary (zed or zed-preview)
    zed_cli_path: Arc<Mutex<Option<String>>>,
}

impl Backend {
    fn new(client: Client) -> Self {
        Self {
            client,
            mnem_client: Arc::new(Mutex::new(None)),
            documents: Arc::new(Mutex::new(HashMap::new())),
            daemon_fail_count: Arc::new(Mutex::new(0)),
            zed_cli_path: Arc::new(Mutex::new(None)),
        }
    }

    /// Detects which Zed binary to use (zed or zed-preview)
    async fn get_zed_cli(&self) -> String {
        let mut cached = self.zed_cli_path.lock().await;
        if let Some(path) = &*cached {
            return path.clone();
        }

        let bins = vec![
            // CLI installed via "Install CLI" command (these are usually symlinks to `cli`)
            "/usr/local/bin/zed-preview",
            "/usr/local/bin/zed",
            "zed-preview",
            "zed",
            // Direct path to CLI within the App Bundle (zed binary is the GUI app, cli is the tool)
            "/Applications/Zed Preview.app/Contents/MacOS/cli",
            "/Applications/Zed.app/Contents/MacOS/cli",
        ];

        for bin in bins {
            if std::path::Path::new(bin).exists()
                || std::process::Command::new("which")
                    .arg(bin)
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false)
            {
                log::info!("Detected Zed CLI: {}", bin);
                *cached = Some(bin.to_string());
                return bin.to_string();
            }
        }

        log::warn!("Could not detect Zed CLI, defaulting to 'zed'");
        "zed".to_string()
    }

    // -----------------------------------------------------------------------
    // Daemon connectivity (with automatic retry)
    // -----------------------------------------------------------------------

    /// Ensures we have a live DaemonClient connection, reconnecting if needed.
    /// Returns false if connection could not be established.
    async fn ensure_daemon(&self) -> bool {
        let mut client_guard = self.mnem_client.lock().await;
        if client_guard.is_some() {
            return true;
        }

        // Exponential back-off: skip attempts if too many failures
        let mut fail_count = self.daemon_fail_count.lock().await;
        if *fail_count > 10 {
            // After 10 consecutive failures, only retry every 10th call
            *fail_count += 1;
            if *fail_count % 10 != 0 {
                return false;
            }
        }

        match DaemonClient::connect() {
            Ok(c) => {
                *client_guard = Some(c);
                *fail_count = 0;
                true
            }
            Err(_) => {
                *fail_count += 1;
                false
            }
        }
    }

    // -----------------------------------------------------------------------
    // Symbol extraction
    // -----------------------------------------------------------------------

    /// Extracts the word/symbol at the given position in the document
    fn extract_symbol_at_position(document_text: &str, position: Position) -> Option<String> {
        let lines: Vec<&str> = document_text.lines().collect();
        if position.line as usize >= lines.len() {
            return None;
        }

        let line = lines[position.line as usize];
        let line_chars: Vec<char> = line.chars().collect();

        if position.character as usize >= line_chars.len() {
            return None;
        }

        let mut start = position.character as usize;
        let mut end = position.character as usize;

        while start > 0 && (line_chars[start - 1].is_alphanumeric() || line_chars[start - 1] == '_')
        {
            start -= 1;
        }

        while end < line_chars.len()
            && (line_chars[end].is_alphanumeric() || line_chars[end] == '_')
        {
            end += 1;
        }

        if start < end {
            Some(line[start..end].to_string())
        } else {
            None
        }
    }

    // -----------------------------------------------------------------------
    // Daemon RPC wrappers
    // -----------------------------------------------------------------------

    /// Gets symbol history from Mnemosyne daemon
    async fn get_symbol_history(&self, symbol_name: &str) -> Result<Vec<SymbolHistoryEntry>> {
        if !self.ensure_daemon().await {
            return Ok(Vec::new());
        }

        let mut client_guard = self.mnem_client.lock().await;
        if let Some(ref mut client) = *client_guard {
            match client.call(
                methods::SYMBOL_GET_HISTORY,
                json!({ "symbol_name": symbol_name }),
            ) {

                Ok(res) => {
                    if let Some(history) = res.get("history") {
                        if let Ok(entries) =
                            serde_json::from_value::<Vec<SymbolHistoryEntry>>(history.clone())
                        {
                            return Ok(entries);
                        }
                    }
                }
                Err(_) => {
                    *client_guard = None;
                }
            }
        }

        Ok(Vec::new())
    }

    /// Gets the semantic diff between two versions of a symbol
    async fn get_symbol_diff(
        &self,
        file_path: &str,
        symbol_name: &str,
        base_hash: Option<&str>,
        target_hash: &str,
    ) -> Result<Option<String>> {
        if !self.ensure_daemon().await {
            return Ok(None);
        }

        let mut client_guard = self.mnem_client.lock().await;
        if let Some(ref mut client) = *client_guard {
            match client.call(
                methods::SYMBOL_GET_DIFF,
                json!({
                    "file_path": file_path,
                    "symbol_name": symbol_name,
                    "base_hash": base_hash,
                    "target_hash": target_hash
                }),
            ) {

                Ok(res) => {
                    if let Some(diff) = res.get("diff").and_then(|d| d.as_str()) {
                        return Ok(Some(diff.to_string()));
                    }
                }
                Err(_) => {
                    *client_guard = None;
                }
            }
        }
        Ok(None)
    }

    /// Gets content for a given hash
    async fn get_snapshot_content(&self, content_hash: &str) -> Result<Option<String>> {
        if !self.ensure_daemon().await {
            return Ok(None);
        }

        let mut client_guard = self.mnem_client.lock().await;
        if let Some(ref mut client) = *client_guard {
            match client.call(
                methods::SNAPSHOT_GET,
                json!({ "content_hash": content_hash }),
            ) {

                Ok(res) => {
                    if let Some(content) = res.get("content").and_then(|c| c.as_str()) {
                        return Ok(Some(content.to_string()));
                    }
                }
                Err(_) => {
                    *client_guard = None;
                }
            }
        }
        Ok(None)
    }

    /// Restores a specific symbol version
    async fn restore_symbol_version(
        &self,
        file_path: &str,
        content_hash: &str,
        symbol_name: &str,
    ) -> Result<bool> {
        if !self.ensure_daemon().await {
            return Ok(false);
        }

        let mut client_guard = self.mnem_client.lock().await;
        if let Some(ref mut client) = *client_guard {
            match client.call(
                methods::SNAPSHOT_RESTORE_SYMBOL_V1,
                json!({
                    "content_hash": content_hash,
                    "target_path": file_path,
                    "symbol_name": symbol_name
                }),
            ) {

                Ok(_) => return Ok(true),
                Err(_) => {
                    *client_guard = None;
                }
            }
        }
        Ok(false)
    }

    // -----------------------------------------------------------------------
    // Hover content formatting
    // -----------------------------------------------------------------------

    /// Formats history information for hover with last change diff
    fn format_hover_content(
        history: &[SymbolHistoryEntry],
        symbol_name: &str,
        current_file: &str,
        diff: Option<String>,
    ) -> String {
        if history.is_empty() {
            return format!("**Mnemosyne**: No history found for `{}`", symbol_name);
        }

        let mut file_snapshots = 0;
        let mut unique_hashes = HashSet::new();

        for entry in history {
            if entry.snapshot.file_path == current_file {
                file_snapshots += 1;
                unique_hashes.insert(&entry.structural_hash);
            }
        }

        let latest = &history[0];
        let branch = latest.snapshot.git_branch.as_deref().unwrap_or("unknown");
        let ts = latest
            .snapshot
            .timestamp
            .split('.')
            .next()
            .unwrap_or(&latest.snapshot.timestamp)
            .replace('T', " ");

        let mut content = format!(
            "**Mnemosyne** | `{}`\n\n\
            | | |\n|---|---|\n\
            | Versions | **{}** ({} unique) |\n\
            | Branch | `{}` |\n\
            | Last Modified | `{}` |\n",
            symbol_name,
            file_snapshots,
            unique_hashes.len(),
            branch,
            ts
        );

        if let Some(diff_text) = diff {
            let trimmed = diff_text.trim();
            if !trimmed.is_empty() {
                content.push_str("\n---\n\n**Last Change**\n");
                content.push_str("```diff\n");
                content.push_str(trimmed);
                content.push_str("\n```\n");
            }
        }

        content.push_str("\n*Use Code Actions (Cmd+.) to view full history or restore a version.*");

        content
    }

    // -----------------------------------------------------------------------
    // Generate history document (Markdown)
    // -----------------------------------------------------------------------

    /// Generates a full Markdown document with all versions and diffs for a symbol
    async fn generate_history_document(
        &self,
        symbol_name: &str,
        file_path: &str,
    ) -> Result<String> {
        let history = self.get_symbol_history(symbol_name).await?;

        let mut doc = String::new();
        doc.push_str(&format!("# History of `{}`\n\n", symbol_name));
        doc.push_str(&format!("**File**: `{}`\n\n", file_path));

        doc.push_str("> [!TIP]\n");
        doc.push_str("> You can use **Code Actions (Cmd+.)** on any symbol to open a **Native Diff View**.\n");
        doc.push_str(
            "> In the diff view, use the **arrows (→)** to selectively restore changes.\n\n",
        );

        // Filter to this file only and deduplicate by structural_hash
        let mut file_entries: Vec<&SymbolHistoryEntry> = history
            .iter()
            .filter(|e| e.snapshot.file_path == file_path)
            .collect();

        if file_entries.is_empty() {
            // Fallback: show all entries if none match the exact path
            file_entries = history.iter().collect();
        }

        // Deduplicate by structural_hash, keeping the earliest timestamp for each version
        let mut seen_hashes: Vec<(String, Vec<&SymbolHistoryEntry>)> = Vec::new();
        for entry in &file_entries {
            if let Some(group) = seen_hashes
                .iter_mut()
                .find(|(h, _)| h == &entry.structural_hash)
            {
                group.1.push(entry);
            } else {
                seen_hashes.push((entry.structural_hash.clone(), vec![entry]));
            }
        }

        doc.push_str(&format!(
            "**Total Snapshots**: {} | **Unique Versions**: {}\n\n",
            file_entries.len(),
            seen_hashes.len()
        ));
        doc.push_str("---\n\n");

        // Show each unique version with its diff from the previous
        let mut previous_hash: Option<&str> = None;

        // Reverse so oldest is first (seen_hashes are ordered newest -> oldest)
        let versions: Vec<_> = seen_hashes.iter().rev().collect();

        for (version_idx, (structural_hash, entries)) in versions.iter().enumerate() {
            let representative = entries.last().unwrap(); // oldest entry in this group
            let ts = representative
                .snapshot
                .timestamp
                .split('.')
                .next()
                .unwrap_or(&representative.snapshot.timestamp)
                .replace('T', " ");
            let branch = representative.snapshot.git_branch.as_deref().unwrap_or("?");

            doc.push_str(&format!(
                "## Version {} | `{}` | branch `{}`\n\n",
                version_idx + 1,
                ts,
                branch
            ));
            doc.push_str(&format!(
                "Hash: `{}...` | Snapshot ID: {}\n\n",
                &structural_hash[..structural_hash.len().min(12)],
                representative.snapshot.id
            ));

            // Show the actual code of this version if we can retrieve it
            if let Ok(Some(content)) = self
                .get_snapshot_content(&representative.snapshot.content_hash)
                .await
            {
                // Try to extract just the symbol from the full file content
                let ext = std::path::Path::new(file_path)
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");
                let lang_hint = match ext {
                    "rs" => "rust",
                    "py" => "python",
                    "ts" | "tsx" => "typescript",
                    "js" | "jsx" => "javascript",
                    "go" => "go",
                    "java" => "java",
                    "c" | "h" => "c",
                    "cpp" | "cxx" | "cc" | "hpp" => "cpp",
                    "cs" => "csharp",
                    "rb" => "ruby",
                    "php" => "php",
                    _ => "",
                };

                // Extract the symbol body from the content using line ranges
                let lines: Vec<&str> = content.lines().collect();
                let start = representative.start_line;
                let end = representative.end_line.min(lines.len());
                if start < end && start < lines.len() {
                    let symbol_code: String = lines[start..end].join("\n");
                    doc.push_str(&format!("```{}\n{}\n```\n\n", lang_hint, symbol_code));
                }
            }

            // Show diff from previous version
            if let Some(prev_h) = previous_hash {
                if prev_h != structural_hash.as_str() {
                    // Get the diff between this version and the previous
                    if let Ok(Some(diff)) = self
                        .get_symbol_diff(
                            file_path,
                            symbol_name,
                            Some(prev_h),
                            &representative.snapshot.content_hash,
                        )
                        .await
                    {
                        let trimmed = diff.trim();
                        if !trimmed.is_empty() {
                            doc.push_str("**Changes from previous version:**\n\n");
                            doc.push_str("```diff\n");
                            doc.push_str(trimmed);
                            doc.push_str("\n```\n\n");
                        }
                    }
                }
            } else {
                doc.push_str("*Initial version*\n\n");
            }

            doc.push_str("---\n\n");
            previous_hash = Some(&representative.snapshot.content_hash);
        }

        Ok(doc)
    }

    /// Writes content to a temp file and returns the file path
    fn write_temp_history_file(symbol_name: &str, content: &str) -> Result<std::path::PathBuf> {
        let temp_dir = std::env::temp_dir().join("mnemosyne-history");
        std::fs::create_dir_all(&temp_dir)?;

        let sanitized = symbol_name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>();
        let filename = format!("{}_history.md", sanitized);
        let path = temp_dir.join(&filename);

        std::fs::write(&path, content)?;

        Ok(path)
    }
}

#[async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> LspResult<InitializeResult> {
        let capabilities = ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            definition_provider: Some(OneOf::Left(true)),
            references_provider: Some(OneOf::Left(true)),
            code_action_provider: Some(CodeActionProviderCapability::Options(CodeActionOptions {
                code_action_kinds: Some(vec![
                    CodeActionKind::new("mnemosyne.history"),
                    CodeActionKind::new("mnemosyne.restore"),
                ]),
                work_done_progress_options: WorkDoneProgressOptions {
                    work_done_progress: None,
                },
                resolve_provider: None,
            })),
            execute_command_provider: Some(ExecuteCommandOptions {
                commands: vec![
                    CMD_VIEW_HISTORY.to_string(),
                    CMD_VIEW_DIFF.to_string(),
                    CMD_RESTORE_SYMBOL.to_string(),
                    CMD_COMPARE_WITH_VERSION.to_string(),
                ],
                work_done_progress_options: WorkDoneProgressOptions {
                    work_done_progress: None,
                },
            }),
            code_lens_provider: Some(CodeLensOptions {
                resolve_provider: Some(false),
            }),
            ..ServerCapabilities::default()
        };

        Ok(InitializeResult {
            capabilities,
            ..InitializeResult::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(
                MessageType::INFO,
                "Mnemosyne LSP initialized. Waiting for daemon connection...",
            )
            .await;
    }

    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Document sync
    // -----------------------------------------------------------------------

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text.clone();
        let mut docs = self.documents.lock().await;
        docs.insert(uri, text);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let mut docs = self.documents.lock().await;
        if let Some(changes) = params.content_changes.first() {
            docs.insert(uri, changes.text.clone());
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let mut docs = self.documents.lock().await;
        docs.remove(&uri);
    }

    // -----------------------------------------------------------------------
    // Hover: shows version count + last diff inline
    // -----------------------------------------------------------------------

    async fn hover(&self, params: HoverParams) -> LspResult<Option<Hover>> {
        let req = params.text_document_position_params.clone();
        let uri = req.text_document.uri.clone();

        let docs = self.documents.lock().await;
        let doc_text = match docs.get(&uri) {
            Some(text) => text.clone(),
            None => return Ok(None),
        };
        drop(docs);

        let symbol = match Self::extract_symbol_at_position(&doc_text, req.position) {
            Some(s) => s,
            None => return Ok(None),
        };

        let file_path = uri
            .to_file_path()
            .ok()
            .and_then(|p| p.to_str().map(|s| s.to_owned()))
            .unwrap_or_default();

        let history = match self.get_symbol_history(&symbol).await {
            Ok(h) => h,
            Err(_) => return Ok(None),
        };

        if history.is_empty() {
            return Ok(None);
        }

        // Compute last semantic diff
        // Compute last semantic diff for THIS file
        let mut diff = None;

        // Filter history to only include entries for the current file
        let file_history: Vec<&SymbolHistoryEntry> = history
            .iter()
            .filter(|e| e.snapshot.file_path == file_path)
            .collect();

        if file_history.len() > 1 {
            let latest_entry = file_history[0];
            let latest_hash = &latest_entry.structural_hash;
            let mut previous_hash = None;

            for entry in file_history.iter().skip(1) {
                if entry.structural_hash != *latest_hash {
                    previous_hash = Some(entry.snapshot.content_hash.as_str());
                    break;
                }
            }

            if let Some(prev) = previous_hash {
                if let Ok(Some(d)) = self
                    .get_symbol_diff(
                        &file_path,
                        &symbol,
                        Some(prev),
                        &latest_entry.snapshot.content_hash,
                    )
                    .await
                {
                    diff = Some(d);
                }
            }
        }

        let content = Self::format_hover_content(&history, &symbol, &file_path, diff);

        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: content,
            }),
            range: None,
        }))
    }

    // -----------------------------------------------------------------------
    // Code Actions: contextual menu for history/restore
    // -----------------------------------------------------------------------

    async fn code_action(&self, params: CodeActionParams) -> LspResult<Option<CodeActionResponse>> {
        let uri = params.text_document.uri.clone();

        let docs = self.documents.lock().await;
        let doc_text = match docs.get(&uri) {
            Some(text) => text.clone(),
            None => return Ok(None),
        };
        drop(docs);

        // Use the start of the selection range
        let position = params.range.start;
        let symbol = match Self::extract_symbol_at_position(&doc_text, position) {
            Some(s) => s,
            None => return Ok(None),
        };

        let file_path = uri
            .to_file_path()
            .ok()
            .and_then(|p| p.to_str().map(|s| s.to_owned()))
            .unwrap_or_default();

        // Only show actions if the symbol has history
        let history = match self.get_symbol_history(&symbol).await {
            Ok(h) if !h.is_empty() => h,
            _ => return Ok(None),
        };

        // Count unique versions for this file
        let unique_versions: HashSet<&String> = history
            .iter()
            .filter(|e| e.snapshot.file_path == file_path)
            .map(|e| &e.structural_hash)
            .collect();

        if unique_versions.len() < 2 {
            // Only one version exists, no point in showing history/restore
            return Ok(None);
        }

        let mut actions: Vec<CodeActionOrCommand> = Vec::new();

        // Action 1: View full history with diffs
        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
            title: format!(
                "Mnemosyne: View {} versions of `{}`",
                unique_versions.len(),
                symbol
            ),
            kind: Some(CodeActionKind::new("mnemosyne.history")),
            command: Some(Command {
                title: "View History".to_string(),
                command: CMD_VIEW_HISTORY.to_string(),
                arguments: Some(vec![json!(symbol), json!(file_path)]),
            }),
            ..CodeAction::default()
        }));

        // Action 2: Restore previous version (find the last different version)
        let latest_hash = &history[0].structural_hash;
        let previous_entry = history
            .iter()
            .filter(|e| e.snapshot.file_path == file_path)
            .find(|e| e.structural_hash != *latest_hash);

        if let Some(prev) = previous_entry {
            let ts = prev
                .snapshot
                .timestamp
                .split('.')
                .next()
                .unwrap_or(&prev.snapshot.timestamp)
                .replace('T', " ");

            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: format!("Mnemosyne: Restore `{}` to version from {}", symbol, ts),
                kind: Some(CodeActionKind::new("mnemosyne.restore")),
                command: Some(Command {
                    title: "Restore Symbol".to_string(),
                    command: CMD_RESTORE_SYMBOL.to_string(),
                    arguments: Some(vec![
                        json!(symbol),
                        json!(file_path),
                        json!(prev.snapshot.content_hash),
                    ]),
                }),
                ..CodeAction::default()
            }));

            // Action 3: Compare with previous version (Zed native diff)
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: format!("Mnemosyne: Compare `{}` with version from {}", symbol, ts),
                kind: Some(CodeActionKind::new("mnemosyne.compare")),
                command: Some(Command {
                    title: "Compare with Version".to_string(),
                    command: CMD_COMPARE_WITH_VERSION.to_string(),
                    arguments: Some(vec![
                        json!(symbol),
                        json!(file_path),
                        json!(prev.snapshot.content_hash),
                    ]),
                }),
                ..CodeAction::default()
            }));
        }

        Ok(Some(actions))
    }

    // -----------------------------------------------------------------------
    // Execute Command: handle custom Mnemosyne commands
    // -----------------------------------------------------------------------

    async fn execute_command(
        &self,
        params: ExecuteCommandParams,
    ) -> LspResult<Option<serde_json::Value>> {
        match params.command.as_str() {
            CMD_VIEW_HISTORY => {
                let args = params.arguments;
                if args.len() < 2 {
                    return Ok(Some(json!({"error": "Missing arguments"})));
                }

                let symbol_name = args[0].as_str().unwrap_or_default();
                let file_path = args[1].as_str().unwrap_or_default();

                self.client
                    .log_message(
                        MessageType::INFO,
                        format!("Generating history for `{}`...", symbol_name),
                    )
                    .await;

                match self.generate_history_document(symbol_name, file_path).await {
                    Ok(content) => {
                        match Self::write_temp_history_file(symbol_name, &content) {
                            Ok(path) => {
                                // Use window/showDocument request (LSP 3.16+) to open the file directly in the editor.
                                // Zed handles this if sent as a request.
                                let url = match Url::from_file_path(&path) {
                                    Ok(u) => u,
                                    Err(_) => {
                                        // Fallback if URL conversion fails
                                        #[cfg(target_os = "macos")]
                                        let _ =
                                            std::process::Command::new("open").arg(&path).spawn();
                                        return Ok(Some(
                                            json!({"status": "ok", "path": path.to_string_lossy()}),
                                        ));
                                    }
                                };

                                let params = ShowDocumentParams {
                                    uri: url,
                                    external: Some(false),
                                    take_focus: Some(true),
                                    selection: None,
                                };

                                if let Err(e) = self
                                    .client
                                    .send_request::<tower_lsp::lsp_types::request::ShowDocument>(
                                        params,
                                    )
                                    .await
                                {
                                    self.client.log_message(
                                        MessageType::LOG,
                                        format!("window/showDocument failed, falling back to system open: {}", e),
                                    ).await;

                                    #[cfg(target_os = "macos")]
                                    let _ = std::process::Command::new("open").arg(&path).spawn();
                                    #[cfg(target_os = "linux")]
                                    let _ =
                                        std::process::Command::new("xdg-open").arg(&path).spawn();
                                    #[cfg(target_os = "windows")]
                                    let _ = std::process::Command::new("cmd")
                                        .args(["/C", "start", &path.to_string_lossy()])
                                        .spawn();
                                }

                                self.client
                                    .show_message(
                                        MessageType::INFO,
                                        format!("History generated: {}", path.display()),
                                    )
                                    .await;

                                Ok(Some(
                                    json!({"status": "ok", "path": path.to_string_lossy()}),
                                ))
                            }
                            Err(e) => {
                                self.client
                                    .show_message(
                                        MessageType::ERROR,
                                        format!("Failed to write history file: {}", e),
                                    )
                                    .await;
                                Ok(Some(json!({"error": e.to_string()})))
                            }
                        }
                    }
                    Err(e) => {
                        self.client
                            .show_message(
                                MessageType::ERROR,
                                format!("Failed to generate history: {}", e),
                            )
                            .await;
                        Ok(Some(json!({"error": e.to_string()})))
                    }
                }
            }

            CMD_VIEW_DIFF => {
                let args = params.arguments;
                if args.len() < 4 {
                    return Ok(Some(json!({"error": "Missing arguments"})));
                }

                let symbol_name = args[0].as_str().unwrap_or_default();
                let file_path = args[1].as_str().unwrap_or_default();
                let base_hash = args[2].as_str();
                let target_hash = args[3].as_str().unwrap_or_default();

                match self
                    .get_symbol_diff(file_path, symbol_name, base_hash, target_hash)
                    .await
                {
                    Ok(Some(diff)) => {
                        let content =
                            format!("# Diff for `{}`\n\n```diff\n{}\n```\n", symbol_name, diff);
                        match Self::write_temp_history_file(
                            &format!("{}_diff", symbol_name),
                            &content,
                        ) {
                            Ok(path) => {
                                // Use window/showDocument request (LSP 3.16+)
                                let url = match Url::from_file_path(&path) {
                                    Ok(u) => u,
                                    Err(_) => {
                                        #[cfg(target_os = "macos")]
                                        let _ =
                                            std::process::Command::new("open").arg(&path).spawn();
                                        return Ok(Some(
                                            json!({"status": "ok", "path": path.to_string_lossy()}),
                                        ));
                                    }
                                };

                                let params = ShowDocumentParams {
                                    uri: url,
                                    external: Some(false),
                                    take_focus: Some(true),
                                    selection: None,
                                };

                                if let Err(e) = self
                                    .client
                                    .send_request::<tower_lsp::lsp_types::request::ShowDocument>(
                                        params,
                                    )
                                    .await
                                {
                                    self.client.log_message(
                                        MessageType::LOG,
                                        format!("window/showDocument failed for diff, falling back to system open: {}", e),
                                    ).await;

                                    #[cfg(target_os = "macos")]
                                    let _ = std::process::Command::new("open").arg(&path).spawn();
                                    #[cfg(target_os = "linux")]
                                    let _ =
                                        std::process::Command::new("xdg-open").arg(&path).spawn();
                                    #[cfg(target_os = "windows")]
                                    let _ = std::process::Command::new("cmd")
                                        .args(["/C", "start", &path.to_string_lossy()])
                                        .spawn();
                                }

                                Ok(Some(
                                    json!({"status": "ok", "path": path.to_string_lossy()}),
                                ))
                            }
                            Err(e) => Ok(Some(json!({"error": e.to_string()}))),
                        }
                    }
                    Ok(None) => Ok(Some(json!({"status": "no_diff"}))),
                    Err(e) => Ok(Some(json!({"error": e.to_string()}))),
                }
            }

            CMD_RESTORE_SYMBOL => {
                let args = params.arguments;
                if args.len() < 3 {
                    return Ok(Some(json!({"error": "Missing arguments"})));
                }

                let symbol_name = args[0].as_str().unwrap_or_default();
                let file_path = args[1].as_str().unwrap_or_default();
                let content_hash = args[2].as_str().unwrap_or_default();

                match self
                    .restore_symbol_version(file_path, content_hash, symbol_name)
                    .await
                {
                    Ok(true) => {
                        self.client
                            .show_message(
                                MessageType::INFO,
                                format!("Restored `{}` successfully.", symbol_name),
                            )
                            .await;
                        Ok(Some(json!({"status": "restored"})))
                    }
                    Ok(false) => {
                        self.client
                            .show_message(
                                MessageType::ERROR,
                                format!("Could not restore `{}`. Daemon unavailable.", symbol_name),
                            )
                            .await;
                        Ok(Some(json!({"error": "daemon_unavailable"})))
                    }
                    Err(e) => {
                        self.client
                            .show_message(MessageType::ERROR, format!("Restore failed: {}", e))
                            .await;
                        Ok(Some(json!({"error": e.to_string()})))
                    }
                }
            }

            CMD_COMPARE_WITH_VERSION => {
                let args = params.arguments;
                if args.len() < 3 {
                    return Ok(Some(json!({"error": "Missing arguments"})));
                }

                let symbol_name = args[0].as_str().unwrap_or_default();
                let file_path = args[1].as_str().unwrap_or_default();
                let content_hash = args[2].as_str().unwrap_or_default();

                log::info!(
                    "Executing CMD_COMPARE_WITH_VERSION for {} in {}",
                    symbol_name,
                    file_path
                );

                // Get the snapshot content
                if let Ok(Some(content)) = self.get_snapshot_content(content_hash).await {
                    let ext = std::path::Path::new(file_path)
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("rs");
                    let hash_short = &content_hash[..8];

                    // Create a temporary file for the base version
                    let temp_dir = std::env::temp_dir().join("mnemosyne-diff");
                    let _ = std::fs::create_dir_all(&temp_dir);
                    let temp_path =
                        temp_dir.join(format!("{}_{}.{}", symbol_name, hash_short, ext));

                    log::info!("Writing temp diff file to: {}", temp_path.display());

                    // Prepend a "Restore Guide" comment because CodeLens isn't showing up
                    let mut file_content = content.clone();
                    if ext == "rs"
                        || ext == "ts"
                        || ext == "js"
                        || ext == "go"
                        || ext == "c"
                        || ext == "cpp"
                        || ext == "java"
                    {
                        let banner = format!("// ------------------------------------------------------------------\n// ↺ TO RESTORE THIS SVERSION:\n// 1. Use the arrows (→) in the center gutter to restore specific blocks\n// 2. OR Run 'Restore Symbol' from the Command Palette (Cmd+Shift+P)\n// ------------------------------------------------------------------\n\n");
                        file_content = format!("{}{}", banner, content);
                    } else if ext == "py" || ext == "sh" {
                        let banner = format!("# ------------------------------------------------------------------\n# ↺ TO RESTORE THIS VERSION:\n# 1. Use the arrows (→) in the center gutter to restore specific blocks\n# 2. OR Run 'Restore Symbol' from the Command Palette (Cmd+Shift+P)\n# ------------------------------------------------------------------\n\n");
                        file_content = format!("{}{}", banner, content);
                    }

                    if let Ok(_) = std::fs::write(&temp_path, file_content) {
                        // Use the detected Zed CLI
                        let zed_bin = self.get_zed_cli().await;
                        log::info!(
                            "Spawning diff: {} --diff {} {}",
                            zed_bin,
                            temp_path.display(),
                            file_path
                        );

                        let spawned = std::process::Command::new(&zed_bin)
                            .arg("--diff")
                            .arg(&temp_path)
                            .arg(file_path)
                            .spawn();

                        match spawned {
                            Ok(_) => {
                                log::info!("Successfully spawned Zed diff");
                                return Ok(Some(json!({"status": "ok"})));
                            }
                            Err(e) => {
                                log::error!("Failed to spawn Zed diff: {}", e);
                                self.client.show_message(
                                    MessageType::ERROR,
                                    format!("Failed to launch '{}': {}. Please ensure Zed CLI is installed.", zed_bin, e)
                                ).await;
                            }
                        }
                    } else {
                        log::error!("Failed to write temp diff file to {}", temp_path.display());
                    }
                } else {
                    log::error!("Failed to get snapshot content for diff: {}", content_hash);
                }

                Ok(Some(json!({"status": "error"})))
            }

            _ => Ok(None),
        }
    }

    // -----------------------------------------------------------------------
    // Go to Definition: shows historical versions as locations
    // -----------------------------------------------------------------------

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> LspResult<Option<GotoDefinitionResponse>> {
        let req = params.text_document_position_params.clone();
        let uri = req.text_document.uri.clone();

        let docs = self.documents.lock().await;
        let doc_text = match docs.get(&uri) {
            Some(text) => text.clone(),
            None => return Ok(None),
        };
        drop(docs);

        let symbol = match Self::extract_symbol_at_position(&doc_text, req.position) {
            Some(s) => s,
            None => return Ok(None),
        };

        let file_path = uri
            .to_file_path()
            .ok()
            .and_then(|p| p.to_str().map(|s| s.to_owned()))
            .unwrap_or_default();

        let history = match self.get_symbol_history(&symbol).await {
            Ok(h) => h,
            Err(_) => return Ok(None),
        };

        let mut locations = Vec::new();
        let mut seen_hashes = HashSet::new();

        for entry in history {
            if entry.snapshot.file_path != file_path {
                continue;
            }

            if !seen_hashes.insert(entry.structural_hash.clone()) {
                continue;
            }

            if let Ok(u) = Url::parse(&format!("file://{}", entry.snapshot.file_path)) {
                locations.push(Location {
                    uri: u,
                    range: Range {
                        start: Position {
                            line: entry.start_line.saturating_sub(1) as u32,
                            character: 0,
                        },
                        end: Position {
                            line: entry.end_line as u32,
                            character: 0,
                        },
                    },
                });
            }

            if locations.len() >= 8 {
                break;
            }
        }

        if locations.is_empty() {
            Ok(None)
        } else {
            Ok(Some(GotoDefinitionResponse::Array(locations)))
        }
    }

    // -----------------------------------------------------------------------
    // References: all historical occurrences across the project
    // -----------------------------------------------------------------------

    async fn references(&self, params: ReferenceParams) -> LspResult<Option<Vec<Location>>> {
        let req = params.text_document_position.clone();
        let uri = req.text_document.uri.clone();

        let docs = self.documents.lock().await;
        let doc_text = match docs.get(&uri) {
            Some(text) => text.clone(),
            None => return Ok(None),
        };
        drop(docs);

        let symbol = match Self::extract_symbol_at_position(&doc_text, req.position) {
            Some(s) => s,
            None => return Ok(None),
        };

        let history = match self.get_symbol_history(&symbol).await {
            Ok(h) => h,
            Err(_) => return Ok(None),
        };

        if history.is_empty() {
            return Ok(None);
        }

        let mut locations = Vec::new();
        let mut seen_states = HashSet::new();

        for entry in history {
            let state_key = format!("{}:{}", entry.snapshot.file_path, entry.structural_hash);
            if !seen_states.insert(state_key) {
                continue;
            }

            if let Ok(u) = Url::parse(&format!("file://{}", entry.snapshot.file_path)) {
                locations.push(Location {
                    uri: u,
                    range: Range {
                        start: Position {
                            line: entry.start_line.saturating_sub(1) as u32,
                            character: 0,
                        },
                        end: Position {
                            line: entry.end_line as u32,
                            character: 0,
                        },
                    },
                });
            }

            if locations.len() >= 20 {
                break;
            }
        }

        if locations.is_empty() {
            Ok(None)
        } else {
            Ok(Some(locations))
        }
    }

    // -----------------------------------------------------------------------
    // Code Lens: file-level snapshot info
    // -----------------------------------------------------------------------

    async fn code_lens(&self, params: CodeLensParams) -> LspResult<Option<Vec<CodeLens>>> {
        let uri = params.text_document.uri.clone();
        let file_path = uri
            .to_file_path()
            .ok()
            .and_then(|p| p.to_str().map(|s| s.to_owned()))
            .unwrap_or_default();

        if file_path.is_empty() {
            return Ok(None);
        }

        if !self.ensure_daemon().await {
            return Ok(None);
        }

        let mut client_guard = self.mnem_client.lock().await;
        if let Some(ref mut client) = *client_guard {
            // Log for debugging
            log::info!("Checking CodeLens for: {}", file_path);

            // --- Special Lens for Mnemosyne History/Diff files ---
            let is_mnem_temp = file_path.contains("mnemosyne-history")
                || file_path.contains("mnemosyne-diff")
                || file_path.contains(".mnemosyne/diffs")
                || file_path.contains("mnem_snap_");

            if is_mnem_temp {
                let lens = CodeLens {
                    range: Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        },
                        end: Position {
                            line: 0,
                            character: 0,
                        },
                    },
                    command: Some(Command {
                        title: "[ ↺ Restore this version ]".to_string(),
                        command: String::new(),
                        arguments: None,
                    }),
                    data: None,
                };
                return Ok(Some(vec![lens]));
            }

            match client.call(methods::FILE_GET_INFO, json!({ "file_path": file_path })) {
                Ok(res) => {

                    let snapshot_count = res
                        .get("snapshot_count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let last_modified = res
                        .get("last_modified")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");

                    if snapshot_count == 0 {
                        return Ok(None);
                    }

                    let time_ago = last_modified.split('T').next().unwrap_or(last_modified);
                    let title = format!(
                        "Mnemosyne: {} snapshots | Last: {} | Cmd+. on symbols for history",
                        snapshot_count, time_ago
                    );

                    let lens = CodeLens {
                        range: Range {
                            start: Position {
                                line: 0,
                                character: 0,
                            },
                            end: Position {
                                line: 0,
                                character: 0,
                            },
                        },
                        command: Some(Command {
                            title,
                            command: String::new(),
                            arguments: None,
                        }),
                        data: None,
                    };

                    return Ok(Some(vec![lens]));
                }
                Err(_) => {
                    *client_guard = None;
                    return Ok(None);
                }
            }
        }

        Ok(None)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Home directory not found"))?;
    let log_dir = home.join(mnem_core::protocol::SOCKET_DIR).join("logs");
    let _ = std::fs::create_dir_all(&log_dir);

    // Initialize logger to write to logs/mnem-lsp.log
    let _ = flexi_logger::Logger::try_with_str("info")?
        .log_to_file(
            flexi_logger::FileSpec::default()
                .directory(&log_dir)
                .basename("mnem-lsp"),
        )
        .start()?;

    log::info!("Starting mnem-lsp server...");

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}
