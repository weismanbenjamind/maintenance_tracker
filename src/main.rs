use clap::Parser;
use maintainence_tracker::MaintenanceTrackerCLI;
use std::process::ExitCode;

fn main() -> ExitCode {
    println!("{:?}", MaintenanceTrackerCLI::parse());
    ExitCode::SUCCESS
}
