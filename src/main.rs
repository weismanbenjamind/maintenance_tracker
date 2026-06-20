use clap::Parser;
use maintenance_tracker::{MaintenanceTrackerArgs, run};
use std::io::{Write, stderr, stdout};
use std::process::ExitCode;

fn main() -> ExitCode {
    match run(MaintenanceTrackerArgs::parse()) {
        Ok(success_msg) => {
            let stdout_ = stdout();
            let mut buf = stdout_.lock();
            writeln!(buf, "{}", success_msg.as_str()).expect("Failed to write result to stdout");
            ExitCode::SUCCESS
        }
        Err(error_msg) => {
            let stderr_ = stderr();
            let mut buf = stderr_.lock();
            writeln!(buf, "{error_msg}").expect("Failed to write result to stderr");
            ExitCode::FAILURE
        }
    }
}
