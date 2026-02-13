use crate::commands::Command;
use crate::ui::Layout;
use anyhow::Result;
use std::env;

#[derive(Debug)]
pub struct GitHookCommand;

impl Command for GitHookCommand {
    fn name(&self) -> &str {
        "git-hook"
    }

    fn usage(&self) -> &str {
        ""
    }

    fn description(&self) -> &str {
        "Install a Git post-commit hook for automatic checkpoints"
    }

    fn group(&self) -> &str {
        "Git Integration"
    }

    fn execute(&self, _args: &[String]) -> Result<()> {
        let layout = Layout::new();

        let cwd = env::current_dir()?;
        let hooks_dir = cwd.join(".git").join("hooks");

        if !hooks_dir.exists() {
            layout.error("Not a git repository (no .git/hooks found)");
            return Ok(());
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

        std::fs::write(&hook_path, hook_content)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&hook_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&hook_path, perms)?;
        }

        layout.success("Git hook installed successfully!");
        layout.item_simple("Snapshots will be automatically linked to Git commits.");

        Ok(())
    }
}
