use anyhow::Result;
use serde::Deserialize;
use std::io::Read;

use crate::ui::Layout;

const GITHUB_REPO: &str = "alessandrobrunoh/Mnemosyne";

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
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

    #[cfg(windows)]
    install_windows(&layout, &client, &release)?;

    #[cfg(unix)]
    install_unix(&layout, &client, &release)?;

    Ok(())
}

#[cfg(windows)]
fn install_windows(
    layout: &crate::ui::Layout,
    client: &reqwest::blocking::Client,
    release: &GitHubRelease,
) -> Result<()> {
    // The Windows release ships as a single zip archive.
    let zip_asset = release
        .assets
        .iter()
        .find(|a| {
            let name = a.name.to_lowercase();
            name.contains("windows") && name.ends_with(".zip")
        })
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Windows zip asset not found in release '{}'. \
                 Expected an asset whose name contains 'windows' and ends with '.zip'.",
                release.tag_name
            )
        })?;

    layout.info(&format!("Downloading {}...", zip_asset.name));

    let zip_bytes = client
        .get(&zip_asset.browser_download_url)
        .send()?
        .bytes()?;

    layout.success_bright("✓ Download complete!");
    layout.empty();

    // Extract mnem.exe and mnem-daemon.exe from the zip into a temp dir.
    let temp_dir = std::env::temp_dir().join("mnemosyne-update");
    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir)?;
    }
    std::fs::create_dir_all(&temp_dir)?;

    let cursor = std::io::Cursor::new(&zip_bytes);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| anyhow::anyhow!("Failed to open zip archive: {}", e))?;

    let required = ["mnem.exe", "mnem-daemon.exe"];
    let mut extracted: Vec<std::path::PathBuf> = Vec::new();

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| anyhow::anyhow!("Failed to read zip entry: {}", e))?;

        let entry_name = entry.name().to_string();
        let file_name = std::path::Path::new(&entry_name)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        if required.contains(&file_name.as_str()) {
            let dest = temp_dir.join(&file_name);
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf)?;
            std::fs::write(&dest, &buf)?;
            layout.info(&format!("Extracted: {}", file_name));
            extracted.push(dest);
        }
    }

    let missing: Vec<&str> = required
        .iter()
        .filter(|&&name| {
            !extracted
                .iter()
                .any(|p| p.file_name().map(|n| n == name).unwrap_or(false))
        })
        .copied()
        .collect();

    if !missing.is_empty() {
        anyhow::bail!(
            "The following binaries were not found in the archive: {}",
            missing.join(", ")
        );
    }

    // Install into ~/.mnemosyne/bin/
    let install_dir = dirs::home_dir()
        .map(|p| p.join(".mnemosyne").join("bin"))
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

    if !install_dir.exists() {
        std::fs::create_dir_all(&install_dir)?;
    }

    // On Windows the running binary cannot be replaced in-place while it is
    // executing.  We write each binary as <name>.new and ask the user to
    // replace them after stopping the daemon.
    for src in &extracted {
        let file_name = src.file_name().unwrap().to_string_lossy();
        let dest_new = install_dir.join(format!("{}.new", file_name));
        std::fs::copy(src, &dest_new)?;
        layout.info(&format!("Staged: {}", dest_new.display()));
    }

    // Clean up temp dir.
    let _ = std::fs::remove_dir_all(&temp_dir);

    layout.empty();
    layout.success_bright("✓ Update staged successfully!");
    layout.empty();
    layout.warning("To complete the update, run the following commands:");
    layout.info("  1. mnem off");
    layout.info("  2. In your install directory, rename each .new file:");
    for name in &required {
        layout.info(&format!("       Rename-Item '{0}.new' '{0}'", name));
    }
    layout.info("  3. mnem on");

    Ok(())
}

#[cfg(unix)]
fn install_unix(
    layout: &crate::ui::Layout,
    client: &reqwest::blocking::Client,
    release: &GitHubRelease,
) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    use std::process::Command;

    let (cli_asset, daemon_asset) = find_unix_assets(&release.assets)?;

    let install_dir = dirs::home_dir()
        .map(|p| p.join(".mnemosyne").join("bin"))
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

    if !install_dir.exists() {
        std::fs::create_dir_all(&install_dir)?;
    }

    let temp_dir = std::env::temp_dir();

    layout.info("Downloading mnem CLI...");
    let cli_bytes = client
        .get(&cli_asset.browser_download_url)
        .send()?
        .bytes()?;
    let cli_tmp = temp_dir.join(&cli_asset.name);
    std::fs::write(&cli_tmp, &cli_bytes)?;

    layout.info("Downloading mnem daemon...");
    let daemon_bytes = client
        .get(&daemon_asset.browser_download_url)
        .send()?
        .bytes()?;
    let daemon_tmp = temp_dir.join(&daemon_asset.name);
    std::fs::write(&daemon_tmp, &daemon_bytes)?;

    layout.success_bright("✓ Download complete!");
    layout.empty();

    // Stop the daemon before replacing the binaries.
    layout.info("Stopping daemon...");
    let current_exe = std::env::current_exe()
        .map_err(|e| anyhow::anyhow!("Could not find current executable: {}", e))?;
    let _ = Command::new(&current_exe).arg("off").output();

    let target_cli = install_dir.join("mnem");
    let target_daemon = install_dir.join("mnem-daemon");

    std::fs::rename(&cli_tmp, &target_cli)?;
    std::fs::rename(&daemon_tmp, &target_daemon)?;
    std::fs::set_permissions(&target_cli, std::fs::Permissions::from_mode(0o755))?;
    std::fs::set_permissions(&target_daemon, std::fs::Permissions::from_mode(0o755))?;

    layout.success_bright("✓ Update installed successfully!");
    layout.empty();
    layout.info("Run 'mnem on' to start the daemon");

    Ok(())
}

#[cfg(unix)]
fn find_unix_assets(assets: &[GitHubAsset]) -> Result<(&GitHubAsset, &GitHubAsset)> {
    let cli_asset = assets
        .iter()
        .find(|a| {
            let name = a.name.to_lowercase();
            // Match standalone "mnem" or "mnem-<version>" but not "mnem-daemon"
            name == "mnem"
                || (name.starts_with("mnem-")
                    && !name.contains("daemon")
                    && !name.ends_with(".zip"))
        })
        .ok_or_else(|| anyhow::anyhow!("CLI asset not found in release"))?;

    let daemon_asset = assets
        .iter()
        .find(|a| {
            let name = a.name.to_lowercase();
            name.contains("mnem") && name.contains("daemon") && !name.ends_with(".zip")
        })
        .ok_or_else(|| anyhow::anyhow!("Daemon asset not found in release"))?;

    Ok((cli_asset, daemon_asset))
}
