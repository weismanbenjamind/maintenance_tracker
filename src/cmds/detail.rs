//! # Detail
//!
//! Get details about a specifc service with a given id.

use clap::Args;
use log::info;

use crate::containers::MaintenanceLog;
use crate::errors::CmdsError;

/// Houses arguments for the detail command.
#[derive(Clone, Debug, Args)]
#[command(about = "Show details about a specific service")]
pub(crate) struct Detail {
    #[arg(help = "ID of service")]
    id: String,
}

impl Detail {
    /// Run the detail command to get info about services with a given id.
    pub(crate) fn run(self, log: &MaintenanceLog) -> Result<String, CmdsError> {
        info!(
            "Getting details for maintenance log item with id {}.",
            self.id
        );

        let metadata = log.get(&self.id)?;
        // Writing to a string cannot fail
        let msg = format!(
            "Id: {}\n\
            {metadata}",
            self.id
        );

        info!("Successfully got details for maintenance log item.");

        Ok(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::containers::{MaintenanceLog, ServiceEvent, ServiceInterval, ServiceMetdata};
    use chrono::NaiveDate;
    use std::collections::HashMap;

    const NAME: &str = "name";
    const SERVICE_INTERVAL: ServiceInterval = ServiceInterval::new(4000, 5);
    const NEXT_SERVICE: ServiceEvent =
        ServiceEvent::new(75000, NaiveDate::from_ymd_opt(2026, 6, 5).unwrap());
    const PREVIOUS_SERVICE: ServiceEvent =
        ServiceEvent::new(71000, NaiveDate::from_ymd_opt(2026, 2, 20).unwrap());
    const NOTE: &str = "Note";
    const ID: &str = "id";

    fn build_metadata() -> ServiceMetdata {
        let previous_services = vec![PREVIOUS_SERVICE];
        let notes = vec![NOTE.to_string()];

        ServiceMetdata::new(
            NAME.into(),
            SERVICE_INTERVAL,
            NEXT_SERVICE,
            Some(previous_services),
            Some(notes),
        )
    }

    fn build_log() -> MaintenanceLog {
        let metadata = build_metadata();
        let mut map: HashMap<String, ServiceMetdata> = HashMap::new();
        map.insert(ID.into(), metadata);

        MaintenanceLog::new(map)
    }

    #[test]
    fn detail_ok() {
        let log = build_log();
        let cmd = Detail { id: ID.into() };
        let found = cmd.run(&log).unwrap();
        let expected = format!("Id: {}\n{}", ID, build_metadata());
        assert_eq!(expected, found);
    }

    #[test]
    fn detail_run_err() {
        let log = build_log();
        let missing_id = "missing_id";
        assert!(!log.contains(missing_id));

        let cmd = Detail {
            id: missing_id.into(),
        };

        let err = cmd.run(&log).unwrap_err();
        match err {
            CmdsError::IdNotFound(_) => (),
            _ => panic!("CmdsError::IdNotFound"),
        }
    }
}
