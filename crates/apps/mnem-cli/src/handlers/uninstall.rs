use anyhow::Result;
use std::process::Command;

use crate::ui::Layout;

pub fn handle_uninstall() -> Result<()> {
    let layout = Layout::new();

    layout.header_dashboard("UNINSTALL MNEMOSYNE");
    layout.warning("This will remove mnem from your system");
    layout.empty();

    let base_dir = dirs::home_dir()
        .map(|p| p.join(".mnemosyne"))
        .unwrap_or_default();

    layout.row_labeled("◫", "Install Dir", &base_dir.to_string_lossy());
    layout.empty();

    layout.info("Running uninstall script...");
    layout.empty();

    #[cfg(windows)]
    {
        let script_path = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .map(|p| p.join("uninstall.ps1"));

        if let Some(script) = script_path {
            if script.exists() {
                let output = Command::new("powershell")
                    .args([
                        "-ExecutionPolicy",
                        "Bypass",
                        "-File",
                        &script.to_string_lossy(),
                    ])
                    .output()?;

                layout.success_bright("✓ Mnemosyne uninstalled successfully");
                layout.info("You can now remove this binary");
            } else {
                layout.warning("Uninstall script not found in binary directory");
                layout.info("Please run the uninstall script manually from the repo:");
                layout.info("  powershell -File scripts/uninstall.ps1");
            }
        } else {
            layout.warning("Could not find uninstall script");
        }
    }

    #[cfg(not(windows))]
    {
        let script_path = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .map(|p| p.join("uninstall.sh"));

        if let Some(script) = script_path {
            if script.exists() {
                let output = Command::new("bash").arg(&script).output()?;

                layout.success_bright("✓ Mnemosyne uninstalled successfully");
                layout.info("You can now remove this binary");
            } else {
                layout.warning("Uninstall script not found");
                layout.info("Please run: bash scripts/uninstall.sh");
            }
        }
    }

    Ok(())
}
