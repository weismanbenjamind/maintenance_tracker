//! # Log
//!
//! Initializes a maintenance log

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::constants::DEFAULT_MAINTENANCE_LOG_PATH;
use crate::containers::{MaintenanceLog, ServiceEvent, ServiceInterval, ServiceMetdata};
use crate::errors::CmdsError;
use chrono::{Local, TimeDelta};
use clap::Args;
use log::{debug, info};

/// Houses arguments for initializing the maintenance log.
#[derive(Clone, Debug, Args)]
#[command(about = "Initializes a maintenance log at a given path")]
pub(crate) struct Log {
    #[arg(
        help = "Path to maintenance log file to be initialized",
        default_value = DEFAULT_MAINTENANCE_LOG_PATH
    )]
    log_path: PathBuf,

    #[arg(
        short,
        long,
        help = "Force an overwrite of a maintenance log if one already exists at the target path"
    )]
    force: bool,
}

impl Log {
    /// Run the Log command to initialize a maintenance log.
    pub(crate) fn run(self) -> Result<String, CmdsError> {
        info!(
            "Attempting to initialize maintenance log at {}.",
            self.log_path.display()
        );

        let msg = match self.log_path.exists() {
            true => handle_existing_log(&self.log_path, self.force),
            false => init_log(&self.log_path),
        }?;

        info!("Maintenance log initialization process complete");
        Ok(msg)
    }
}

/// Handles a situation where a maintenance log already exists.
/// If the force argument is true the log will be created.
/// If the force argument is false the user will be informed to pass the --force arg
/// to overwrite the maintenance log.
#[inline]
fn handle_existing_log(path: &Path, force: bool) -> Result<String, CmdsError> {
    info!(
        "Maintenance log exists at {}. Handling with force={force}.",
        path.display()
    );
    match force {
        true => init_log(path),
        false => {
            debug!("Informing user to use --force arg.");
            let msg = format!(
                "Maintenance log already exists at {}. Rerun with --force (-f) to override the current maintenance log.",
                path.display()
            );
            Ok(msg)
        }
    }
}

/// Initializes a maintenance log at the given path.
fn init_log(path: &Path) -> Result<String, CmdsError> {
    info!("Writing example maintenance log to {}", path.display());

    let time_delta = TimeDelta::weeks(13);
    let today = Local::now().date_naive();
    let next_service_date = today + time_delta;
    let next_service_miles = 78000;

    let name = "Example Service";
    let service_interval = ServiceInterval::new(5000, 6);
    let next_service = ServiceEvent::new(next_service_miles, next_service_date);

    let prev_service_1_date = today - time_delta;
    let prev_service_2_date = prev_service_1_date - TimeDelta::weeks(26);
    let prev_service_1_miles = 73000;
    let prev_service_2_miles = 70000;
    let previous_services = Some(vec![
        ServiceEvent::new(prev_service_1_miles, prev_service_1_date),
        ServiceEvent::new(prev_service_2_miles, prev_service_2_date),
    ]);

    let notes = Some([
        "This is an example service",
        "Fill out this log with your own services in a similar fashion to this example",
        "Can use 'maintenance_log init' to initialize services with proper scaffolding",
        "Notes are optional",
        "Previous services are optional",
        "Everything else must be filled out (e.g. name, next service, service interval, and an id defined by [services.id] in the .toml file)",
        "Once your services have been initialized delete this example",
    ].iter().map(|note| note.to_string()).collect());

    let service_metadata = ServiceMetdata::new(
        name,
        service_interval,
        next_service,
        previous_services,
        notes,
    );

    let mut services: HashMap<String, ServiceMetdata> = HashMap::with_capacity(1);
    services.insert("example_service".into(), service_metadata);

    MaintenanceLog::new(services)
        .write(path)
        .map_err(CmdsError::from)?;

    let msg = format!(
        "Successfully initialized maintenance log at {}",
        path.display()
    );

    info!(
        "Successfully wrote example maintenance log to {}",
        path.display()
    );

    Ok(msg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn log_test_handle_non_existing_log_force() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("log.toml");
        assert!(!path.exists());

        let found = handle_existing_log(&path, true).unwrap();

        assert_eq!(
            found,
            format!(
                "Successfully initialized maintenance log at {}",
                path.display()
            )
        );
        assert!(path.exists());
        assert!(!std::fs::read_to_string(path).unwrap().is_empty());
    }

    #[test]
    fn log_test_handle_existing_log_force() {
        let temp_file = NamedTempFile::with_suffix(".toml").unwrap();
        let path = temp_file.path();
        assert!(path.exists());
        assert!(std::fs::read_to_string(path).unwrap().is_empty());

        let found = handle_existing_log(path, true).unwrap();

        assert_eq!(
            found,
            format!(
                "Successfully initialized maintenance log at {}",
                path.display()
            )
        );
        assert!(!std::fs::read_to_string(path).unwrap().is_empty());
    }

    #[test]
    fn log_test_handle_existing_log_no_force() {
        let found = handle_existing_log(&PathBuf::from("some_path"), false).unwrap();
        assert!(found.to_lowercase().contains("rerun with --force (-f)"))
    }

    #[test]
    fn log_init_log() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("log.toml");
        assert!(!path.exists());

        let found = init_log(&path).unwrap();

        assert_eq!(
            found,
            format!(
                "Successfully initialized maintenance log at {}",
                path.display()
            )
        );
        assert!(path.exists());
        assert!(
            std::fs::read_to_string(path)
                .unwrap()
                .to_lowercase()
                .contains("this is an example service")
        );
    }
}
