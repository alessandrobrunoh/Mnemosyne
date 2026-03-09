use anyhow::Result;
use std::process::Command;

use crate::ui::Layout;
use crate::ui::presentable::SimpleResponse;
use crate::ui::Presentable;

pub fn handle_uninstall(purge: bool, json: bool) -> Result<()> {
    let layout = Layout::new();

    if !json {
        layout.header_dashboard("UNINSTALL MNEMOSYNE");
        if purge {
            layout.warning("This will remove mnem and ALL configuration/history from your system");
        } else {
            layout.warning("This will remove mnem binaries from your system (history will be preserved)");
        }
        layout.empty();
    }

    let base_dir = dirs::home_dir()
        .map(|p| p.join(".mnemosyne"))
        .unwrap_or_default();

    if !json {
        layout.row_labeled("◫", "Install Dir", &base_dir.to_string_lossy());
        layout.empty();
        layout.info("Running uninstall script...");
        layout.empty();
    }

    let result = run_uninstall_script(purge)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&result.render_json()?)?);
    } else {
        if result.success {
            layout.success_bright(&format!("✓ {}", result.message));
            layout.info("You can now remove this binary");
        } else {
            layout.error(&result.message);
            if result.code == Some("UNINSTALL_SCRIPT_NOT_FOUND".to_string()) {
                #[cfg(windows)]
                layout.info("Please run: powershell -File scripts/uninstall.ps1");
                #[cfg(not(windows))]
                layout.info("Please run: bash scripts/uninstall.sh");
            }
        }
    }

    Ok(())
}

#[cfg(windows)]
fn run_uninstall_script(purge: bool) -> Result<SimpleResponse> {
    // Try multiple locations for the uninstall script
    let script_locations = vec![
        // Binary directory
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .map(|p| p.join("uninstall.ps1")),
        // Install directory
        dirs::home_dir().map(|p| p.join(".mnemosyne").join("bin").join("uninstall.ps1")),
        // Current working directory
        std::env::current_dir()
            .ok()
            .map(|p| p.join("scripts").join("uninstall.ps1")),
    ];

    let script_path = script_locations.into_iter().flatten().find(|p| p.exists());

    if let Some(script) = script_path {
        let mut args = vec![
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            script.to_str().unwrap_or_default(),
        ];
        if purge {
            args.push("--purge");
        }

        let output = Command::new("powershell")
            .args(&args)
            .output()?;

        if output.status.success() {
            Ok(SimpleResponse {
                success: true,
                message: "Mnemosyne uninstalled successfully".to_string(),
                code: Some("UNINSTALL_SUCCESS".to_string()),
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Ok(SimpleResponse {
                success: false,
                message: format!("Uninstall script failed: {}", stderr.trim()),
                code: Some("UNINSTALL_SCRIPT_FAILED".to_string()),
            })
        }
    } else {
        Ok(SimpleResponse {
            success: false,
            message: "Uninstall script not found".to_string(),
            code: Some("UNINSTALL_SCRIPT_NOT_FOUND".to_string()),
        })
    }
}

#[cfg(not(windows))]
fn run_uninstall_script(purge: bool) -> Result<SimpleResponse> {
    // Try multiple locations for the uninstall script
    let script_locations = vec![
        // Binary directory
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .map(|p| p.join("uninstall.sh")),
        // Install directory
        dirs::home_dir().map(|p| p.join(".mnemosyne").join("bin").join("uninstall.sh")),
        // Current working directory
        std::env::current_dir()
            .ok()
            .map(|p| p.join("scripts").join("uninstall.sh")),
    ];

    let script_path = script_locations.into_iter().flatten().find(|p| p.exists());

    if let Some(script) = script_path {
        let mut cmd = Command::new("bash");
        cmd.arg(&script);
        if purge {
            cmd.arg("--purge");
        }
        let output = cmd.output()?;

        if output.status.success() {
            Ok(SimpleResponse {
                success: true,
                message: "Mnemosyne uninstalled successfully".to_string(),
                code: Some("UNINSTALL_SUCCESS".to_string()),
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Ok(SimpleResponse {
                success: false,
                message: format!("Uninstall script failed: {}", stderr.trim()),
                code: Some("UNINSTALL_SCRIPT_FAILED".to_string()),
            })
        }
    } else {
        Ok(SimpleResponse {
            success: false,
            message: "Uninstall script not found".to_string(),
            code: Some("UNINSTALL_SCRIPT_NOT_FOUND".to_string()),
        })
    }
}
