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

    // Determine binary names based on OS
    let (cli_name, daemon_name) = if cfg!(target_os = "windows") {
        ("mnem.exe", "mnem-daemon.exe")
    } else {
        ("mnem", "mnem-daemon")
    };

    // Download CLI - preserve original filename from asset
    layout.info("Downloading mnem CLI...");
    let cli_response = client.get(&cli_asset.browser_download_url).send()?;
    let cli_bytes = cli_response.bytes()?;
    let cli_path = temp_dir.join(&cli_asset.name);
    std::fs::write(&cli_path, &cli_bytes)?;

    // Download Daemon
    layout.info("Downloading mnem daemon...");
    let daemon_response = client.get(&daemon_asset.browser_download_url).send()?;
    let daemon_bytes = daemon_response.bytes()?;
    let daemon_path = temp_dir.join(&daemon_asset.name);
    std::fs::write(&daemon_path, &daemon_bytes)?;

    layout.success_bright("✓ Download complete!");
    layout.empty();

    // Replace binaries with OS-appropriate names
    let target_cli = install_dir.join(cli_name);
    let target_daemon = install_dir.join(daemon_name);

    // On Windows, download to .new names to avoid file locking
    // User will need to restart manually
    #[cfg(windows)]
    {
        let new_cli = install_dir.join(format!("{}.new", cli_name));
        let new_daemon = install_dir.join(format!("{}.new", daemon_name));

        // Copy to .new files
        std::fs::copy(&cli_path, &new_cli)?;
        std::fs::copy(&daemon_path, &new_daemon)?;

        layout.success_bright("✓ Update downloaded!");
        layout.empty();
        layout.warning("Please restart mnem to complete the update:");
        layout.info(&format!("  1. Stop daemon: mnem off"));
        layout.info(&format!("  2. Replace {} with {}.new", cli_name, cli_name));
        layout.info(&format!(
            "  3. Replace {} with {}.new",
            daemon_name, daemon_name
        ));
        layout.info("  4. Run 'mnem on' to start");
    }

    // On Unix, stop daemon and replace directly
    #[cfg(unix)]
    {
        // Stop daemon
        layout.info("Stopping daemon...");
        let _ = Command::new(&current_exe).arg("off").output();

        use std::os::unix::fs::PermissionsExt;
        std::fs::rename(&cli_path, &target_cli)?;
        std::fs::rename(&daemon_path, &target_daemon)?;
        std::fs::set_permissions(&target_cli, std::fs::Permissions::from_mode(0o755))?;
        std::fs::set_permissions(&target_daemon, std::fs::Permissions::from_mode(0o755))?;

        layout.success_bright("✓ Update installed successfully!");
        layout.empty();
        layout.info("Run 'mnem on' to start the daemon");
    }

    Ok(())
}

fn find_assets(assets: &[GitHubAsset]) -> Result<(&GitHubAsset, &GitHubAsset)> {
    let cli_asset = assets
        .iter()
        .find(|a| {
            let name = a.name.to_lowercase();
            name.contains("mnem")
                && !name.contains("daemon")
                && (name.ends_with(".exe") || name == "mnem" || name.starts_with("mnem-"))
        })
        .ok_or_else(|| anyhow::anyhow!("CLI asset not found"))?;

    let daemon_asset = assets
        .iter()
        .find(|a| {
            let name = a.name.to_lowercase();
            name.contains("mnem") && name.contains("daemon")
        })
        .ok_or_else(|| anyhow::anyhow!("Daemon asset not found"))?;

    Ok((cli_asset, daemon_asset))
}
