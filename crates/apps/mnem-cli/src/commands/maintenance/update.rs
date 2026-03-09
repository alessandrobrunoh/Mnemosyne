use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::ui::Layout;
use serde_json::Value;
use crate::ui::Presentable;

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

#[derive(Serialize)]
pub struct UpdateResponse {
    pub success: bool,
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub message: String,
    pub check_only: bool,
}

impl Presentable for UpdateResponse {
    fn render_tui(&self) -> Result<()> {
        let layout = Layout::new();
        layout.header_dashboard("UPDATE STATUS");
        layout.row_labeled("◆", "Current Version", &self.current_version);
        layout.row_labeled("◆", "Latest Version", &self.latest_version);
        layout.empty();

        if self.update_available {
            layout.warning(&format!("New version available: v{}", self.latest_version));
            if self.check_only {
                layout.info("Run 'mnem update' to install the new version");
            } else if self.success {
                layout.success_bright("✓ Update installed successfully!");
                layout.info("Run 'mnem on' to start the daemon");
            } else {
                layout.error(&self.message);
            }
        } else {
            layout.success_bright("✓ You are on the latest version!");
        }
        Ok(())
    }

    fn render_json(&self) -> Result<Value> {
        Ok(serde_json::to_value(self)?)
    }
}

pub fn handle_update(check_only: bool, json: bool) -> Result<()> {
    let layout = Layout::new();

    if !json {
        layout.header_dashboard("CHECKING FOR UPDATES");
        layout.empty();
    }

    let current_version = env!("CARGO_PKG_VERSION");
    
    if !json {
        layout.row_labeled("◆", "Current Version", current_version);
        layout.empty();
        layout.info("Checking GitHub for latest release...");
        layout.empty();
    }

    let client = reqwest::blocking::Client::builder()
        .user_agent("Mnemosyne-CLI")
        .build()?;

    let url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        GITHUB_REPO
    );
    let response = client.get(&url).send()?;

    if !response.status().is_success() {
        if json {
            println!("{}", serde_json::json!({
                "success": false,
                "error": "Could not check for updates",
                "http_status": response.status().as_u16()
            }));
        } else {
            layout.warning("Could not check for updates");
            layout.info(&format!("GitHub API returned: {}", response.status()));
        }
        return Ok(());
    }

    let release: GitHubRelease = response.json()?;
    let latest_version = release.tag_name.trim_start_matches('v');
    let update_available = latest_version != current_version;

    let mut update_res = UpdateResponse {
        success: true,
        current_version: current_version.to_string(),
        latest_version: latest_version.to_string(),
        update_available,
        message: if update_available { "New version available".to_string() } else { "Already on latest version".to_string() },
        check_only,
    };

    if !update_available || check_only {
        if json {
            println!("{}", serde_json::to_string_pretty(&update_res.render_json()?)?);
        } else {
            update_res.render_tui()?;
        }
        return Ok(());
    }

    let install_result = perform_installation(&layout, &client, &release, json);

    match install_result {
        Ok(_) => {
            update_res.success = true;
            update_res.message = "Update installed successfully".to_string();
        }
        Err(e) => {
            update_res.success = false;
            update_res.message = format!("Installation failed: {}", e);
        }
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&update_res.render_json()?)?);
    } else {
        update_res.render_tui()?;
    }

    Ok(())
}

fn perform_installation(
    layout: &Layout,
    client: &reqwest::blocking::Client,
    release: &GitHubRelease,
    json: bool,
) -> Result<()> {
    #[cfg(windows)]
    {
        install_windows(layout, client, release, json)
    }
    #[cfg(not(windows))]
    {
        install_unix(layout, client, release, json)
    }
}

#[cfg(windows)]
fn install_windows(
    layout: &crate::ui::Layout,
    client: &reqwest::blocking::Client,
    release: &GitHubRelease,
    json: bool,
) -> Result<()> {
    use std::io::Read;
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

    if !json {
        layout.info(&format!("Downloading {}...", zip_asset.name));
    }

    let zip_bytes = client
        .get(&zip_asset.browser_download_url)
        .send()?
        .bytes()?;

    if !json {
        layout.success_bright("✓ Download complete!");
        layout.empty();
    }

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
            if !json {
                layout.info(&format!("Extracted: {}", file_name));
            }
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
        if !json {
            layout.info(&format!("Staged: {}", dest_new.display()));
        }
    }

    // Clean up temp dir.
    let _ = std::fs::remove_dir_all(&temp_dir);

    if !json {
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
    }

    Ok(())
}

#[cfg(not(windows))]
fn install_unix(
    layout: &crate::ui::Layout,
    client: &reqwest::blocking::Client,
    release: &GitHubRelease,
    json: bool,
) -> Result<()> {
    use std::io::Read;
    use std::os::unix::fs::PermissionsExt;
    use std::process::Command;

    let zip_asset = find_unix_zip_asset(&release.assets)?;

    let install_dir = dirs::home_dir()
        .map(|p| p.join(".mnemosyne").join("bin"))
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

    if !install_dir.exists() {
        std::fs::create_dir_all(&install_dir)?;
    }

    if !json {
        layout.info(&format!("Downloading {}...", zip_asset.name));
    }
    let zip_bytes = client
        .get(&zip_asset.browser_download_url)
        .send()?
        .bytes()?;

    if !json {
        layout.success_bright("✓ Download complete!");
        layout.empty();
    }

    // Extract mnem and mnem-daemon from the zip into a temp dir.
    let temp_dir = std::env::temp_dir().join("mnemosyne-update");
    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir)?;
    }
    std::fs::create_dir_all(&temp_dir)?;

    let cursor = std::io::Cursor::new(&zip_bytes);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| anyhow::anyhow!("Failed to open zip archive: {}", e))?;

    let required = ["mnem", "mnem-daemon"];
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
            if !json {
                layout.info(&format!("Extracted: {}", file_name));
            }
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

    // Stop the daemon before replacing the binaries.
    if !json {
        layout.info("Stopping daemon...");
    }
    let current_exe = std::env::current_exe()
        .map_err(|e| anyhow::anyhow!("Could not find current executable: {}", e))?;
    let _ = Command::new(&current_exe).arg("off").output();

    let target_cli = install_dir.join("mnem");
    let target_daemon = install_dir.join("mnem-daemon");

    std::fs::rename(temp_dir.join("mnem"), &target_cli)?;
    std::fs::rename(temp_dir.join("mnem-daemon"), &target_daemon)?;
    std::fs::set_permissions(&target_cli, std::fs::Permissions::from_mode(0o755))?;
    std::fs::set_permissions(&target_daemon, std::fs::Permissions::from_mode(0o755))?;

    let _ = std::fs::remove_dir_all(&temp_dir);

    if !json {
        layout.success_bright("✓ Update installed successfully!");
        layout.empty();
        layout.info("Run 'mnem on' to start the daemon");
    }

    Ok(())
}

#[cfg(not(windows))]
fn find_unix_zip_asset(assets: &[GitHubAsset]) -> Result<&GitHubAsset> {
    let platform = if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    };

    let arch = if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "x86_64"
    };

    // Expected name: mnem-<platform>-<arch>.zip  e.g. mnem-macos-arm64.zip
    assets
        .iter()
        .find(|a| {
            let name = a.name.to_lowercase();
            name.starts_with("mnem-")
                && name.contains(platform)
                && name.contains(arch)
                && name.ends_with(".zip")
        })
        .ok_or_else(|| {
            anyhow::anyhow!(
                "ZIP asset not found in release for {}-{}. Expected mnem-{}-{}.zip",
                platform,
                arch,
                platform,
                arch
            )
        })
}
