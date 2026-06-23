//! # Run
//!
//! Main entry point for maintenance log library.
//! Meant to be called by `main` in maintenance log binary.

use crate::cfg_resolve::resolve_cfg;
use crate::cli::Cmd;
use crate::cli::MaintenanceTrackerArgs;
use crate::containers::MaintenanceLog;
use crate::errors::ContainersError;
use crate::errors::MaintenanceTrackerError;
use crate::verbosity::set_verbosity;
use log::info;

const RUST_LOG: &str = "RUST_LOG";

pub struct SuccessMsg(String);

impl SuccessMsg {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub fn run(args: MaintenanceTrackerArgs) -> Result<SuccessMsg, MaintenanceTrackerError> {
    let (maintenance_log, maintenance_log_env, verbose, cmd) = args.into_parts();

    let rust_log = std::env::var(RUST_LOG).ok();
    set_verbosity(verbose, rust_log)?;
    info!("Starting maintenance tracking run.");

    if let Cmd::Log(log_cmd) = cmd {
        let msg = log_cmd.run().map_err(MaintenanceTrackerError::from)?;
        return Ok(SuccessMsg(msg));
    }

    let maintenance_log_path = resolve_cfg(&maintenance_log, &maintenance_log_env)?;
    let mut log = MaintenanceLog::load(&maintenance_log_path).map_err(ContainersError::from)?;

    let msg = match cmd {
        Cmd::List(list_cmd) => list_cmd.run(&log)?,
        Cmd::Next(next_cmd) => next_cmd.run(&log)?,
        Cmd::Init(init_cmd) => init_cmd.run(&mut log)?,
        Cmd::Detail(detail_cmd) => detail_cmd.run(&log)?,
        Cmd::Complete(complete_cmd) => complete_cmd.run(&mut log)?,
        Cmd::Update(update_cmd) => update_cmd.run(&mut log)?,
        Cmd::Delete(delete_cmd) => delete_cmd.run(&mut log)?,
        Cmd::Diff(diff_cmd) => diff_cmd.run(&log)?,
        Cmd::Status(status_cmd) => status_cmd.run(&log)?,
        Cmd::Log(_) => {
            return Err(MaintenanceTrackerError::InvalidState(
                "Log command not valid.".into(),
            ));
        }
    };

    log.write(&maintenance_log_path)
        .map_err(ContainersError::from)?;

    info!("Maintenance tracking run complete.");

    Ok(SuccessMsg(msg))
}
