//! # CLI
//!
//! Top level command line interface for `maintenance_log` library and binary.

use crate::cmds;
use crate::constants::DEFAULT_MAINTENANCE_LOG_PATH;
use std::path::PathBuf;

use clap::{ArgAction, Parser, Subcommand};

/// Top level args for the maintenance_log CLI.
#[derive(Clone, Debug, Parser)]
#[command(
    name = "maintenance_tracker",
    about = "CLI tool to track auto maintenance",
    version
)]
pub struct MaintenanceTrackerArgs {
    #[arg(
        short,
        long,
        default_value = DEFAULT_MAINTENANCE_LOG_PATH,
        help = "Path to maintenance log"
    )]
    maintenance_log: PathBuf,

    #[arg(
        short = 'e',
        long,
        default_value = "MAINTENANCE_LOG",
        help = "Environment variable which can be used to find the maintainance log if the value specified by the --maintenance-log cannot be found. Pass skip to disable attempting to use this variable"
    )]
    maintenance_log_env: String,

    #[arg(short, long, action = ArgAction::Count, help = "Verbosity. Pass -v for info and -vv for debug. Anything after -vv will set the verbosity to debug. Defaults to warning")]
    verbose: u8,

    #[command(subcommand)]
    cmd: Cmd,
}

impl MaintenanceTrackerArgs {
    /// Consumes the MaintenanceTrackerArgs object returning a tuple in the format
    /// `PathBuf`, `String`, `u8`, `Cmd`
    ///
    /// Where:
    /// - `PathBuf` => `MaintenanceTrackerArgs.maintenance_log`
    /// - `String` => `MaintenanceTrackerArgs.maintenance_log_env`
    /// - `u8` => `MaintenanceTrackerArgs.verbose`
    /// - `Cmd` => `MaintenanceTrackerArgs.cmd`
    pub(crate) fn into_parts(self) -> (PathBuf, String, u8, Cmd) {
        (
            self.maintenance_log,
            self.maintenance_log_env,
            self.verbose,
            self.cmd,
        )
    }
}

/// Enum to house all potential top level commands for the `maintenance_log` CLI.
#[derive(Clone, Debug, Subcommand)]
pub(crate) enum Cmd {
    Complete(cmds::Complete),
    Delete(cmds::Delete),
    Detail(cmds::Detail),
    Diff(cmds::Diff),
    Init(cmds::Init),
    List(cmds::List),
    Log(cmds::Log),
    Next(cmds::Next),
    Status(cmds::Status),
    Update(cmds::Update),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::matches;

    #[test]
    fn test_cli_into_parts() {
        let maintenance_log = PathBuf::from("path");
        let maintenance_log_env = "env".to_string();
        let verbose: u8 = 1;
        let cmd = Cmd::List(cmds::List);

        let args = MaintenanceTrackerArgs {
            maintenance_log: maintenance_log.clone(),
            maintenance_log_env: maintenance_log_env.clone(),
            verbose,
            cmd,
        };

        let (found_maintenance_log, found_maintenance_log_env, found_verbose, found_cmd) =
            args.into_parts();

        assert_eq!(maintenance_log, found_maintenance_log);
        assert_eq!(maintenance_log_env, found_maintenance_log_env);
        assert_eq!(verbose, found_verbose);
        assert!(matches!(found_cmd, Cmd::List(_)));
    }
}
