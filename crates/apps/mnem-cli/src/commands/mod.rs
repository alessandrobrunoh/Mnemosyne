use clap::Subcommand;

pub mod common;
pub mod daemon;
pub mod files;
pub mod general;
pub mod maintenance;
pub mod workspace;

use common::{CommandStrategy, GlobalOptions};

// Import all migrated command structs
use daemon::{
    McpStartCommand, McpStatusCommand, McpStopCommand, OffCommand, OnCommand, StatusCommand,
};
use files::{HistoryCommand, InfoCommand, RestoreCommand, SCommand};
use general::GitCommand;
use maintenance::{ConfigCommand, GcCommand, UninstallCommand, UpdateCommand};
use workspace::TrackCommand;

/// Macro to generate the Commands enum and execute() method
///
/// This macro reduces boilerplate by automatically generating:
/// - The Commands enum with struct variants
/// - The execute() method that dispatches to the appropriate command
macro_rules! declare_commands {
    ($($variant_name:ident => $struct_name:ident),* $(,)?) => {
        #[derive(Subcommand)]
        pub enum Commands {
            $(
                $variant_name($struct_name),
            )*
        }

        impl Commands {
            /// Execute the command with the given global options
            pub fn execute(&self, global_opts: &GlobalOptions) -> anyhow::Result<()> {
                match self {
                    $(
                        Commands::$variant_name(cmd) => cmd.execute(global_opts),
                    )*
                }
            }
        }
    };
}

// Declare all migrated commands using the macro
declare_commands! {
    // Daemon commands
    On => OnCommand,
    Off => OffCommand,
    Status => StatusCommand,
    McpStart => McpStartCommand,
    McpStop => McpStopCommand,
    McpStatus => McpStatusCommand,

    // File commands
    History => HistoryCommand,
    Info => InfoCommand,
    Restore => RestoreCommand,
    S => SCommand,

    // Maintenance commands
    Gc => GcCommand,
    Config => ConfigCommand,
    Git => GitCommand,
    Uninstall => UninstallCommand,
    Update => UpdateCommand,

    // Workspace commands
    Track => TrackCommand,
}
