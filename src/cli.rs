use crate::cmds;
use std::path::{Path, PathBuf};

use clap::{ArgAction, Parser, Subcommand};

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
        default_value = "maintenance_log.toml",
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
    pub(crate) fn maintenance_log(&self) -> &Path {
        &self.maintenance_log
    }

    pub(crate) fn maintenance_log_env(&self) -> &str {
        &self.maintenance_log_env
    }

    pub(crate) fn verbose(&self) -> u8 {
        self.verbose
    }

    pub(crate) fn into_cmd(self) -> Cmd {
        self.cmd
    }
}

#[derive(Clone, Debug, Subcommand)]
pub(crate) enum Cmd {
    Complete(cmds::Complete),
    Delete(cmds::Delete),
    Detail(cmds::Detail),
    Diff(cmds::Diff),
    Init(cmds::Init),
    List(cmds::List),
    Next(cmds::Next),
    Status(cmds::Status),
    Update(cmds::Update),
}
