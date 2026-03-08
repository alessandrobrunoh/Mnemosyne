use anyhow::Result;
use mnem_core::{client::DaemonClient, protocol::methods};

pub fn handle_gc(keep: Option<usize>, dry_run: bool, _aggressive: bool) -> Result<()> {
    use crate::ui::Layout;
    use crossterm::style::Stylize;

    let layout = Layout::new();
    layout.section_start("gc", "Garbage Collection");

    if dry_run {
        layout.item_simple("Dry run - no changes will be made");
        layout.section_end();
        return Ok(());
    }

    match DaemonClient::connect() {
        Ok(mut client) => {
            let params = serde_json::json!({
                "keep_days": keep,
            });

            let res = client.call(methods::MAINTENANCE_GC, params)?;
            let pruned = res["pruned"].as_u64().unwrap_or(0);

            layout.item_simple(&format!(
                "{} Successfully pruned {} orphan chunks.",
                "√".green(),
                pruned.to_string().bold()
            ));
        }
        Err(_) => {
            layout.error("Daemon is not running. Start it with 'mnem on'");
        }
    }

    layout.section_end();
    Ok(())
}
