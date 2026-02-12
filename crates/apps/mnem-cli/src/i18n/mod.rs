pub mod en;
pub mod it;

pub enum Locale {
    En,
    It,
}

impl Locale {
    pub fn from_env() -> Self {
        match std::env::var("LANG").unwrap_or_default().as_str() {
            s if s.starts_with("it") => Locale::It,
            _ => Locale::En,
        }
    }
}

pub trait Messages {
    fn app_name(&self) -> &'static str;
    fn tagline(&self) -> &'static str;
    fn core_ops_header(&self) -> &'static str;

    // Commands
    fn cmd_default_desc(&self) -> &'static str;
    fn cmd_start_desc(&self) -> &'static str;
    fn cmd_stop_desc(&self) -> &'static str;

    // Project History
    fn project_history_header(&self) -> &'static str;
    fn cmd_list_desc(&self) -> &'static str;
    fn cmd_log_desc(&self) -> &'static str;
    fn cmd_search_desc(&self) -> &'static str;

    // Maintenance
    fn maintenance_header(&self) -> &'static str;
    fn cmd_status_desc(&self) -> &'static str;
    fn cmd_config_desc(&self) -> &'static str;

    // Footer
    fn learn_more(&self) -> &'static str;
}

pub fn current() -> Box<dyn Messages> {
    match Locale::from_env() {
        Locale::En => Box::new(en::English),
        Locale::It => Box::new(it::Italian),
    }
}
