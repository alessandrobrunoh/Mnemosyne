use zed_extension_api::{self as zed, LanguageServerId};

struct MnemosyneExtension;

impl zed::Extension for MnemosyneExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        // Cerca il binario `mnem-lsp` nel PATH dell'ambiente di lavoro.
        // Nelle versioni future potremo implementare il download automatico dei binari precompilati.
        let path = worktree.which("mnem-lsp").ok_or_else(|| {
            "Il binario 'mnem-lsp' non è stato trovato nel PATH. \
                 Assicurati che sia installato e accessibile per permettere a Mnemosyne \
                 di fornire la storia locale e la navigazione semantica."
                .to_string()
        })?;

        Ok(zed::Command {
            command: path,
            args: Vec::new(),
            env: Vec::new(),
        })
    }

    fn language_server_initialization_options(
        &mut self,
        _language_server_id: &LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> zed::Result<Option<zed::serde_json::Value>> {
        // Forniamo opzioni di inizializzazione per abilitare esplicitamente le feature dell'LSP
        Ok(Some(zed::serde_json::json!({
            "provide_history_on_hover": true,
            "provide_historical_goto": true,
            "semantic_analysis_enabled": true
        })))
    }
}

zed::register_extension!(MnemosyneExtension);
