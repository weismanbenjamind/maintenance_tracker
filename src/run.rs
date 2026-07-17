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

/// New type to wrap a string to print a run success message.
pub struct SuccessMsg(String);

impl SuccessMsg {
    /// Get the message as a borred string.
    /// Only used for testing.
    #[cfg(test)]
    fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SuccessMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Run the maintenance tracker.
pub fn run(args: MaintenanceTrackerArgs) -> Result<SuccessMsg, MaintenanceTrackerError> {
    let (maintenance_log, maintenance_log_env, verbose, cmd) = args.into_parts();

    let rust_log = std::env::var(RUST_LOG).ok();
    set_verbosity(verbose, rust_log);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmds::{self, NextService, ServiceInterval};
    use crate::testing::build_log;
    use crate::testing::constants::{ID, NOTE};

    use chrono::NaiveDate;
    use tempfile::{NamedTempFile, TempDir};

    fn init_log_file() -> NamedTempFile {
        let tmpfile = NamedTempFile::with_suffix(".toml").unwrap();
        let log = build_log();
        log.write(&tmpfile).unwrap();
        tmpfile
    }

    #[test]
    fn run_complete_cmd() {
        todo!("Write this test");
    }

    #[test]
    fn run_detail_cmd() {
        let tmpfile = init_log_file();

        let args = MaintenanceTrackerArgs::new(
            tmpfile.path().into(),
            None,
            None,
            Cmd::Detail(cmds::Detail::new(ID)),
        );

        let result = run(args).unwrap();
        assert!(result.as_str().to_lowercase().starts_with("id: "))
    }

    #[test]
    fn run_init_cmd() {
        let tmpfile = init_log_file();
        let new_id = "new_id";
        let new_note = "New Note".to_string();

        assert_ne!(new_id, ID);
        assert_ne!(&new_note, NOTE);
        let contents = std::fs::read_to_string(&tmpfile).unwrap();
        assert!(!contents.contains(new_id));
        assert!(!contents.contains(&new_note));

        let args = MaintenanceTrackerArgs::new(
            tmpfile.path().into(),
            None,
            None,
            Cmd::Init(cmds::Init::new(
                "new_name",
                new_id,
                ServiceInterval::new(4000, 5),
                NextService::new(74000, NaiveDate::from_ymd_opt(2026, 6, 15).unwrap()),
                Some(vec!["New Note".to_string()]),
                None,
            )),
        );

        let result = run(args).unwrap();
        assert!(
            result
                .as_str()
                .to_lowercase()
                .contains("initialized the following service:")
        );
        let contents = std::fs::read_to_string(tmpfile).unwrap();
        assert!(contents.contains(new_id));
        assert!(contents.contains(&new_note));
    }

    #[test]
    fn run_next_cmd() {
        let tmpfile = init_log_file();
        let args = MaintenanceTrackerArgs::new(
            tmpfile.path().into(),
            None,
            None,
            Cmd::Next(cmds::Next::new(ID)),
        );

        let result = run(args).unwrap();
        assert!(
            result
                .as_str()
                .to_lowercase()
                .contains("next service miles")
        )
    }

    #[test]
    fn run_list_cmd() {
        let tmpfile = init_log_file();

        let args =
            MaintenanceTrackerArgs::new(tmpfile.path().into(), None, None, Cmd::List(cmds::List));
        let result = run(args).unwrap();
        assert!(result.to_string().to_lowercase().contains(ID))
    }

    #[test]
    fn run_log_cmd() {
        let tmp_dir = TempDir::new().unwrap();
        let log_path = tmp_dir.path().join("log.toml");
        assert!(!log_path.exists());

        let args = MaintenanceTrackerArgs::new(
            None,
            None,
            None,
            Cmd::Log(cmds::Log::new(&log_path, false)),
        );

        let msg = run(args).unwrap();
        assert!(
            msg.as_str()
                .to_lowercase()
                .contains("successfully initialized maintenance log")
        );
        assert!(log_path.exists())
    }

    #[test]
    fn run_success_msg() {
        let msg = String::from("success");
        let new_type = SuccessMsg(msg.clone());
        assert_eq!(format!("{new_type}"), msg);
        assert_eq!(new_type.as_str(), &msg)
    }
}
