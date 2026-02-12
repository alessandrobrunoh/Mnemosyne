use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn install_hook(project_root: &Path) -> Result<()> {
    let hooks_dir = project_root.join(".git").join("hooks");
    if !hooks_dir.exists() {
        return Err(anyhow::anyhow!(
            "Not a git repository (no .git/hooks found)"
        ));
    }

    let hook_path = hooks_dir.join("post-commit");

    let hook_content = r#"#!/bin/sh
# Mnemosyne post-commit hook
# Links the latest snapshots to the official Git commit

if command -v mnem >/dev/null 2>&1; then
    COMMIT_HASH=$(git rev-parse HEAD)
    AUTHOR=$(git log -1 --pretty=%an)
    MESSAGE=$(git log -1 --pretty=%s)
    TIMESTAMP=$(git log -1 --pretty=%cI)
    
    mnem git-event "$COMMIT_HASH" "$MESSAGE" "$AUTHOR" "$TIMESTAMP"
else
    echo "Warning: Mnemosyne (mnem) not found in PATH. Skipping integration."
fi
"#;

    fs::write(&hook_path, hook_content)?;

    crate::os::set_executable(&hook_path)?;

    Ok(())
}
