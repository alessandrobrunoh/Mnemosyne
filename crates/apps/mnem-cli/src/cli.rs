/// Parsed CLI arguments
#[derive(Debug, Clone)]
pub struct Args {
    /// Raw command line arguments (including program name at index 0)
    pub raw: Vec<String>,
    /// Command name (e.g., "log", "diff", "tui")
    pub command: Option<String>,
    /// Positional arguments
    pub positional: Vec<String>,
    /// Flag arguments (e.g., "--verbose", "-v")
    pub flags: Vec<String>,
}

impl Args {
    /// Parse command line arguments
    pub fn parse(raw: Vec<String>) -> Self {
        let command = if raw.len() > 1 {
            Some(raw[1].clone())
        } else {
            None
        };

        let mut positional = Vec::new();
        let mut flags = Vec::new();

        // Skip program name (index 0) and command (index 1) if present
        let start_idx = if command.is_some() { 2 } else { 1 };

        for arg in raw.iter().skip(start_idx) {
            if arg.starts_with('-') {
                flags.push(arg.clone());
            } else {
                positional.push(arg.clone());
            }
        }

        Self {
            raw,
            command,
            positional,
            flags,
        }
    }

    /// Check if a flag is present
    pub fn has_flag(&self, flag: &str) -> bool {
        self.flags.contains(&flag.to_string())
    }

    /// Get a flag value (e.g., "--limit 10" returns Some("10"))
    pub fn flag_value(&self, flag: &str) -> Option<&String> {
        let pos = self.flags.iter().position(|f| f == flag)?;
        self.raw.iter().skip(pos + 2).next()
    }

    /// Get a positional argument by index
    pub fn get(&self, index: usize) -> Option<&String> {
        self.positional.get(index)
    }
}

/// Common command options that appear across multiple commands
#[derive(Debug, Clone)]
pub struct CommonOptions {
    /// Limit the number of results
    pub limit: Option<usize>,
    /// Filter by branch
    pub branch: Option<String>,
    /// Verbose output
    pub verbose: bool,
    /// Dry run (don't make changes)
    pub dry_run: bool,
}

impl CommonOptions {
    /// Parse common options from args
    pub fn from_args(args: &Args) -> Self {
        let limit = args
            .flag_value("--limit")
            .or_else(|| args.flag_value("-l"))
            .and_then(|v| v.parse::<usize>().ok());

        let branch = args
            .flag_value("--branch")
            .or_else(|| args.flag_value("-b"))
            .cloned();

        let verbose = args.has_flag("--verbose") || args.has_flag("-v");
        let dry_run = args.has_flag("--dry-run") || args.has_flag("-n");

        Self {
            limit,
            branch,
            verbose,
            dry_run,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_parsing() {
        let raw = vec![
            "mnem".to_string(),
            "log".to_string(),
            "--limit".to_string(),
            "10".to_string(),
            "file.txt".to_string(),
        ];

        let args = Args::parse(raw);

        assert_eq!(args.command, Some("log".to_string()));
        assert_eq!(
            args.positional,
            vec!["10".to_string(), "file.txt".to_string()]
        );
        assert_eq!(args.flags, vec!["--limit".to_string()]);
    }

    #[test]
    fn test_no_command() {
        let raw = vec!["mnem".to_string()];
        let args = Args::parse(raw);

        assert_eq!(args.command, None);
        assert!(args.positional.is_empty());
        assert!(args.flags.is_empty());
    }

    #[test]
    fn test_common_options() {
        let raw = vec![
            "mnem".to_string(),
            "log".to_string(),
            "--limit".to_string(),
            "10".to_string(),
            "--verbose".to_string(),
            "--branch".to_string(),
            "main".to_string(),
        ];

        let args = Args::parse(raw);
        let opts = CommonOptions::from_args(&args);

        assert_eq!(opts.limit, Some(10));
        assert_eq!(opts.branch, Some("main".to_string()));
        assert!(opts.verbose);
    }
}
