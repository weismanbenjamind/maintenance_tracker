//! # Next
//!
//! Command to get a specific service's next miles and date thresholds.

use clap::Args;

use crate::containers::MaintenanceLog;
use crate::errors::CmdsError;
use log::info;

/// Houses the arguments for the next command.
#[derive(Clone, Debug, Args)]
#[command(about = "Get the next service event for a given maintenance item")]
pub(crate) struct Next {
    #[arg(help = "ID of service")]
    id: String,
}

impl Next {
    /// Run the next command. Get's a specific service's next miles and date thresholds
    /// as a human readible string.
    pub(crate) fn run(self, log: &MaintenanceLog) -> Result<String, CmdsError> {
        info!("Getting next service for id {}.", self.id);
        let found = log.get(&self.id)?;

        let next_service = found.next_service();

        let msg = format!(
            "Next Service Miles: {}\n\
            Next Service Date: {}",
            next_service.miles(),
            next_service.date()
        );

        info!("Finished getting next service for id {}.", self.id);
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
    fn test_next_run_ok() {
        let expected = format!(
            "Next Service Miles: {}\n\
            Next Service Date: {}",
            NEXT_SERVICE.miles(),
            NEXT_SERVICE.date()
        );
        let cmd = Next { id: ID.into() };
        let found = cmd.run(&build_log()).unwrap();
        assert_eq!(expected, found);
    }

    #[test]
    fn test_next_run_id_not_found() {
        let id = "not found";
        assert_ne!(ID, id);

        let cmd = Next { id: id.into() };
        let found = cmd.run(&build_log()).unwrap_err();
        match found {
            CmdsError::IdNotFound(_) => (),
            _ => panic!("Expected CmdsError::IdNotFound"),
        }
    }
}
