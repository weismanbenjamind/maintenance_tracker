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
    pub fn new(maintenance_log: &Path, maintenance_log_env: &str, verbose: u8, cmd: Cmd) -> Self {
        Self {
            maintenance_log: maintenance_log.into(),
            maintenance_log_env: maintenance_log_env.into(),
            verbose,
            cmd,
        }
    }

    pub fn maintenance_log(&self) -> &Path {
        &self.maintenance_log
    }

    pub fn maintenance_log_env(&self) -> &str {
        &self.maintenance_log_env
    }

    pub fn verbose(&self) -> u8 {
        self.verbose
    }

    pub fn cmd(&self) -> &Cmd {
        &self.cmd
    }
}

#[derive(Clone, Debug, Subcommand)]
pub enum Cmd {
    Complete(cmds::Complete),
    Detail(cmds::Detail),
    Diff(cmds::Diff),
    Init(cmds::Init),
    List(cmds::List),
    Next(cmds::Next),
    Update(cmds::Update),
}

impl Cmd {
    pub fn run(self) {}
}
