use anyhow::Result;
use serde::Serialize;
use serde_json::Value;

/// Trait per gli oggetti che possono essere presentati sia in formato TUI che JSON.
pub trait Presentable {
    /// Rendering dell'output testuale/grafico (TUI).
    fn render_tui(&self) -> Result<()>;

    /// Trasformazione dell'output in un oggetto JSON per l'AI.
    fn render_json(&self) -> Result<Value>;
}

/// Una risposta semplice per comandi che restituiscono solo un messaggio di stato.
#[derive(Serialize)]
pub struct SimpleResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

impl Presentable for SimpleResponse {
    fn render_tui(&self) -> Result<()> {
        use crate::ui::Layout;
        let layout = Layout::new();
        if self.success {
            layout.badge_success("SUCCESS", &self.message);
        } else {
            layout.badge_error("ERROR", &self.message);
        }
        Ok(())
    }

    fn render_json(&self) -> Result<Value> {
        Ok(serde_json::to_value(self)?)
    }
}
