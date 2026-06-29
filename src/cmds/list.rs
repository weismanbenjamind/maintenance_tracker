//! # List
//!
//! Command to list all possible service interval ids.

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
    use crate::testing::{build_log, build_metadata, constants};

    #[test]
    fn list_run() {
        let mut log = build_log();
        let new_metadata = build_metadata();
        let new_id = "new_id";
        assert!(!log.contains(new_id));
        assert!(log.insert(new_id, new_metadata).is_none());

        let cmd = List;
        let result = cmd.run(&log).unwrap();
        let expected = format!("{}\n{}", constants::ID, new_id);
        assert_eq!(result, expected);
    }
}
