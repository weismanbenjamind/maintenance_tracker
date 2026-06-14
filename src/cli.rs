use crate::cmds;
use crate::constants::DEFAULT_MAINTENANCE_LOG_PATH;
use std::path::PathBuf;

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
    pub(crate) fn into_parts(self) -> (PathBuf, String, u8, Cmd) {
        (
            self.maintenance_log,
            self.maintenance_log_env,
            self.verbose,
            self.cmd,
        )
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
    Log(cmds::Log),
    Next(cmds::Next),
    Status(cmds::Status),
    Update(cmds::Update),
}
