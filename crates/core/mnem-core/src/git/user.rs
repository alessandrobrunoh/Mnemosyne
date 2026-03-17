//! Git user resolver for attributing code changes to specific users
//!
//! This module provides functionality to read Git configuration files
//! and extract user information (name and email) for attribution purposes.

use crate::error::{AppError, AppResult};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Git user information extracted from Git configuration
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitUser {
    pub name: String,
    pub email: String,
}

impl GitUser {
    /// Create a new GitUser from name and email
    pub fn new(name: String, email: String) -> Self {
        Self { name, email }
    }

    /// Format as "Name <email@example.com>"
    pub fn display(&self) -> String {
        format!("{} <{}>", self.name, self.email)
    }

    /// Check if user information is complete (both name and email present)
    pub fn is_complete(&self) -> bool {
        !self.name.is_empty() && !self.email.is_empty()
    }
}

impl std::fmt::Display for GitUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} <{}>", self.name, self.email)
    }
}

/// Resolver for Git user information with caching
///
/// This resolver reads Git configuration from multiple sources:
/// 1. Project-level `.git/config`
/// 2. Global `~/.gitconfig`
///
/// Results are cached per repository path for performance.
pub struct GitUserResolver {
    cache: HashMap<PathBuf, Option<GitUser>>,
}

impl GitUserResolver {
    /// Create a new resolver with an empty cache
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Resolve Git user information for a repository path
    ///
    /// Resolution strategy:
    /// 1. Check cache for previously resolved user
    /// 2. Read project-level `.git/config`
    /// 3. Fallback to global `~/.gitconfig`
    /// 4. Return None if no user information is found
    ///
    /// # Arguments
    /// * `repo_path` - Path to the Git repository
    ///
    /// # Returns
    /// * `Ok(Some(GitUser))` - If user information is found
    /// * `Ok(None)` - If repository exists but no user info is configured
    /// * `Err(AppError)` - If repository path is invalid or unreadable
    pub fn resolve(&mut self, repo_path: &Path) -> AppResult<Option<GitUser>> {
        // Normalize path for consistent cache keys
        let canonical = repo_path
            .canonicalize()
            .map_err(|e| AppError::InvalidPath(format!("Cannot canonicalize path: {}", e)))?;

        // Check cache first
        if let Some(cached) = self.cache.get(canonical.as_path()) {
            return Ok(cached.clone());
        }

        // Try project-level config first
        let project_config = repo_path.join(".git/config");
        let user = if project_config.exists() {
            self.read_config(&project_config)?
        } else {
            // Fallback to global config
            if let Some(home) = dirs::home_dir() {
                let global_config = home.join(".gitconfig");
                self.read_config(&global_config)?
            } else {
                None
            }
        };

        // Cache the result (even if None, to avoid repeated failed lookups)
        self.cache.insert(canonical, user.clone());

        Ok(user)
    }

    /// Read and parse a Git configuration file
    ///
    /// # Arguments
    /// * `config_path` - Path to the Git config file
    ///
    /// # Returns
    /// * `Ok(Some(GitUser))` - If user section is found and complete
    /// * `Ok(None)` - If file doesn't exist or user section is incomplete
    /// * `Err(AppError)` - If file cannot be read
    fn read_config(&self, config_path: &Path) -> AppResult<Option<GitUser>> {
        if !config_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(config_path).map_err(|source| AppError::Io {
            path: config_path.to_path_buf(),
            source,
        })?;

        Ok(self.parse_git_config(&content))
    }

    /// Parse Git config file format and extract user section
    ///
    /// Git config format (simplified):
    /// ```text
    /// [user]
    ///     name = John Doe
    ///     email = john@example.com
    /// ```
    ///
    /// # Arguments
    /// * `content` - The contents of the Git config file
    ///
    /// # Returns
    /// * `Some(GitUser)` - If both name and email are found
    /// * `None` - If user section is incomplete or missing
    fn parse_git_config(&self, content: &str) -> Option<GitUser> {
        let mut in_user_section = false;
        let mut name = String::new();
        let mut email = String::new();

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Check for [user] section
            if line == "[user]" {
                in_user_section = true;
                continue;
            }

            // Exit user section when we hit another section
            if line.starts_with('[') && line != "[user]" {
                if in_user_section {
                    break; // We've left the [user] section
                }
            }

            // Parse user entries
            if in_user_section {
                if let Some(value) = parse_config_value(line, "name") {
                    name = value;
                } else if let Some(value) = parse_config_value(line, "email") {
                    email = value;
                }
            }
        }

        // Only return GitUser if both name and email are present
        if !name.is_empty() && !email.is_empty() {
            Some(GitUser::new(name, email))
        } else {
            None
        }
    }

    /// Clear the internal cache
    ///
    /// Useful for testing or when Git configuration changes
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Preload user information for a specific repository
    ///
    /// This can be useful to warm up the cache before bulk operations
    pub fn preload(&mut self, repo_path: &Path) -> AppResult<Option<&GitUser>> {
        self.resolve(repo_path)?;
        Ok(self.cache.get(repo_path).and_then(|opt| opt.as_ref()))
    }
}

impl Default for GitUserResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to parse a Git config key-value line
///
/// # Arguments
/// * `line` - The line to parse (e.g., "    name = John Doe")
/// * `key` - The key to look for (e.g., "name")
///
/// # Returns
/// * `Some(value)` - If the key matches and value is extracted
/// * `None` - If the key doesn't match or line is malformed
fn parse_config_value(line: &str, key: &str) -> Option<String> {
    // Expected format: key = value
    // Note: Git config allows flexible whitespace
    let line = line.trim();

    if !line.starts_with(key) {
        return None;
    }

    // Check for " = " separator
    let separator = format!("{} =", key);
    if !line.starts_with(&separator) {
        return None;
    }

    // Extract value after " = "
    let value = line
        .strip_prefix(&separator)?
        .trim()
        .trim_start_matches('=')
        .trim()
        .to_string();

    Some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config_value() {
        assert_eq!(
            parse_config_value("name = John Doe", "name"),
            Some("John Doe".to_string())
        );
        assert_eq!(
            parse_config_value("email=john@example.com", "email"),
            Some("john@example.com".to_string())
        );
        assert_eq!(parse_config_value("name = ", "name"), Some("".to_string()));
        assert_eq!(parse_config_value("other = value", "name"), None);
    }

    #[test]
    fn test_parse_git_config_complete() {
        let config = r#"
[user]
    name = Jane Doe
    email = jane@example.com

[core]
    repositoryformatversion = 0
"#;

        let resolver = GitUserResolver::new();
        let user = resolver.parse_git_config(config);

        assert_eq!(
            user,
            Some(GitUser::new(
                "Jane Doe".to_string(),
                "jane@example.com".to_string()
            ))
        );
    }

    #[test]
    fn test_parse_git_config_incomplete() {
        let config = r#"
[user]
    name = Only Name
"#;

        let resolver = GitUserResolver::new();
        let user = resolver.parse_git_config(config);

        assert_eq!(user, None);
    }

    #[test]
    fn test_git_user_display() {
        let user = GitUser::new("Mario Rossi".to_string(), "mario@example.com".to_string());
        assert_eq!(user.display(), "Mario Rossi <mario@example.com>");
    }

    #[test]
    fn test_git_user_is_complete() {
        let complete = GitUser::new("Name".to_string(), "email@example.com".to_string());
        assert!(complete.is_complete());

        let incomplete_name = GitUser::new(String::new(), "email@example.com".to_string());
        assert!(!incomplete_name.is_complete());

        let incomplete_email = GitUser::new("Name".to_string(), String::new());
        assert!(!incomplete_email.is_complete());
    }
}
