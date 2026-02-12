use super::Messages;

pub struct English;

impl Messages for English {
    fn app_name(&self) -> &'static str {
        "MNEMOSYNE CLI"
    }
    fn tagline(&self) -> &'static str {
        "The local history companion for developers."
    }
    fn core_ops_header(&self) -> &'static str {
        "CORE OPERATIONS"
    }

    fn cmd_default_desc(&self) -> &'static str {
        "Launch the interactive TUI search and timeline"
    }
    fn cmd_start_desc(&self) -> &'static str {
        "Ensure background daemon is running"
    }
    fn cmd_stop_desc(&self) -> &'static str {
        "Shutdown the background daemon"
    }

    fn project_history_header(&self) -> &'static str {
        "PROJECT HISTORY"
    }
    fn cmd_list_desc(&self) -> &'static str {
        "List all projects currently tracked"
    }
    fn cmd_log_desc(&self) -> &'static str {
        "Show history/snapshots for a specific file"
    }
    fn cmd_search_desc(&self) -> &'static str {
        "Global grep across all project history"
    }

    fn maintenance_header(&self) -> &'static str {
        "MAINTENANCE"
    }
    fn cmd_status_desc(&self) -> &'static str {
        "Check background daemon health and activity"
    }
    fn cmd_config_desc(&self) -> &'static str {
        "Modify tool settings (retention, compression)"
    }

    fn learn_more(&self) -> &'static str {
        "To learn more, check the docs"
    }
}
