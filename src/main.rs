use clap::Parser;
use maintenance_tracker::{MaintenanceTrackerArgs, run};
use std::process::ExitCode;

fn main() -> ExitCode {
    match run(MaintenanceTrackerArgs::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
