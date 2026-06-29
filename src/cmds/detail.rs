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
    use crate::testing::constants::ID;
    use crate::testing::{build_log, build_metadata};

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
