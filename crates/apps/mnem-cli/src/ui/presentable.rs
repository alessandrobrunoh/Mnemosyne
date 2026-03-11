use anyhow::Result;
use serde::Serialize;
use serde_json::Value;

/// Trait per gli oggetti che possono essere renderizzati in formato testuale o JSON.
pub trait Renderable {
    /// Rendering dell'output testuale/terminale.
    fn text(&self) -> Result<()>;

    /// Trasformazione dell'output in JSON.
    /// Implementazione di default che usa Serialize.
    fn json(&self) -> Result<Value>
    where
        Self: Serialize,
    {
        Ok(serde_json::to_value(self)?)
    }
}

/// Una risposta semplice per comandi che restituiscono solo un messaggio di stato.
#[derive(Serialize)]
pub struct SimpleResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

impl Renderable for SimpleResponse {
    fn text(&self) -> Result<()> {
        use crate::ui::Layout;
        let layout = Layout::new();
        if self.success {
            layout.badge_success("SUCCESS", &self.message);
        } else {
            layout.badge_error("ERROR", &self.message);
        }
        Ok(())
    }
}
