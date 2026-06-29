//! # List
//!
//! Command to list all possible service interval ids

use crate::containers::MaintenanceLog;
use crate::errors::CmdsError;
use clap::Args;
use log::info;
use std::fmt::Write;

// Using a unit (static) struct hre to follow all command patters
/// Struct to house args for listing all service interval ids
#[derive(Clone, Copy, Debug, Args)]
#[command(about = "List all services and their ids")]
pub(crate) struct List;

impl List {
    // Need to return a cmds error because in main we return a maintenance log error
    // Errors can only bubble up one level ?
    // So return the level right below MaintenanceLogError - which is Cmds Error
    // Also this function technically cannot fail but return a result to be consistent with other .run() methods
    // ^ Makes error handling easier
    /// Run the list command to get a string back which lays out all ids currently present.
    /// Cannot fail but returns a result for consistency with other .run methods off commands.
    pub(crate) fn run(self, log: &MaintenanceLog) -> Result<String, CmdsError> {
        info!("Writing maintenance log ids to stdout.");

        // Keep ids sorted alphabetically - have to collect
        let mut ids: Vec<&String> = log.ids().collect();
        ids.sort();

        let mut buf = String::new();
        // Writing to a string can't fail so unwrap
        ids.iter().for_each(|id| writeln!(buf, "{id}").unwrap());

        info!("Successfully wrote maintenance log ids to stdout.");
        Ok(buf.trim_end().into())
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
    fn list_run() {
        let mut log = build_log();
        let new_metadata = build_metadata();
        let new_id = "new_id";
        assert!(!log.contains(new_id));
        assert!(log.insert(new_id, new_metadata).is_none());

        let cmd = List;
        let result = cmd.run(&log).unwrap();
        let expected = format!("{}\n{}", ID, new_id);
        assert_eq!(result, expected);
    }
}
