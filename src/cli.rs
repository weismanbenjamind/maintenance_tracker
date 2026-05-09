use std::path::PathBuf;

use clap::{ArgAction, Parser};

#[derive(Clone, Debug, Parser)]
#[command(
    name = "maintenance_tracker",
    about = "CLI tool to track auto maintainence",
    version
)]
pub struct MaintenanceTrackerCLI {
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

    #[arg(short, long, action = ArgAction::Count, help = "Verbosity. Pass -v for info and -vv for debug. Anything after -vv will set the verbosity to debug.")]
    verbose: u8,
}
