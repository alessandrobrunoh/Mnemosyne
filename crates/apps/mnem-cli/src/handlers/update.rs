use anyhow::Result;
use serde::Deserialize;
use std::process::Command;

use crate::ui::Layout;

const GITHUB_REPO: &str = "alessandrobrunoh/Mnemosyne";

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

pub fn handle_update(check_only: bool) -> Result<()> {
    let layout = Layout::new();

    layout.header_dashboard("CHECKING FOR UPDATES");
    layout.empty();

    let current_version = env!("CARGO_PKG_VERSION");
    layout.row_labeled("◆", "Current Version", current_version);
    layout.empty();

    layout.info("Checking GitHub for latest release...");
    layout.empty();

    let client = reqwest::blocking::Client::builder()
        .user_agent("Mnemosyne-CLI")
        .build()?;

    let url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        GITHUB_REPO
    );
    let response = client.get(&url).send()?;

    if !response.status().is_success() {
        layout.warning("Could not check for updates");
        layout.info(&format!("GitHub API returned: {}", response.status()));
        return Ok(());
    }

    let release: GitHubRelease = response.json()?;
    let latest_version = release.tag_name.trim_start_matches('v');

    layout.row_labeled("◆", "Latest Version", latest_version);
    layout.empty();

    if latest_version == current_version {
        layout.success_bright("✓ You are on the latest version!");
        layout.empty();
        return Ok(());
    }

    layout.warning(&format!("New version available: v{}", latest_version));
    layout.empty();

    if check_only {
        layout.info("Run 'mnem update' to install the new version");
        return Ok(());
    }

    let (cli_asset, daemon_asset) = find_assets(&release.assets)?;

    layout.info("Downloading update...");
    layout.empty();

    let install_dir = dirs::home_dir()
        .map(|p| p.join(".mnemosyne").join("bin"))
        .unwrap_or_default();

    if !install_dir.exists() {
        std::fs::create_dir_all(&install_dir)?;
    }

    let current_exe = std::env::current_exe()
        .map_err(|e| anyhow::anyhow!("Could not find current executable: {}", e))?;

    let temp_dir = std::env::temp_dir();

    // Download CLI
    layout.info("Downloading mnem CLI...");
    let cli_response = client.get(&cli_asset.browser_download_url).send()?;
    let cli_bytes = cli_response.bytes()?;
    let cli_path = temp_dir.join("mnem-new.exe");
    std::fs::write(&cli_path, &cli_bytes)?;

    // Download Daemon
    layout.info("Downloading mnem daemon...");
    let daemon_response = client.get(&daemon_asset.browser_download_url).send()?;
    let daemon_bytes = daemon_response.bytes()?;
    let daemon_path = temp_dir.join("mnem-daemon-new.exe");
    std::fs::write(&daemon_path, &daemon_bytes)?;

    layout.success_bright("✓ Download complete!");
    layout.empty();

    // Stop daemon first
    layout.info("Stopping daemon...");
    let _ = Command::new(&current_exe).arg("off").output();

    // Replace binaries
    let target_cli = install_dir.join("mnem.exe");
    let target_daemon = install_dir.join("mnem-daemon.exe");

    std::fs::rename(&cli_path, &target_cli)?;
    std::fs::rename(&daemon_path, &target_daemon)?;

    layout.success_bright("✓ Update installed successfully!");
    layout.empty();
    layout.info("Run 'mnem on' to start the daemon");

    Ok(())
}

fn find_assets(assets: &[GitHubAsset]) -> Result<(&GitHubAsset, &GitHubAsset)> {
    let cli_asset = assets
        .iter()
        .find(|a| a.name == "mnem.exe")
        .ok_or_else(|| anyhow::anyhow!("CLI asset not found"))?;

    let daemon_asset = assets
        .iter()
        .find(|a| a.name == "mnem-daemon.exe")
        .ok_or_else(|| anyhow::anyhow!("Daemon asset not found"))?;

    Ok((cli_asset, daemon_asset))
}
