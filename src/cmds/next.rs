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
    // TODO - Test this
    /// Create a new next struct.
    /// Only used for testing.
    #[cfg(test)]
    pub(crate) fn new(id: &str) -> Self {
        Self { id: id.into() }
    }

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
    use crate::testing::constants::ID;
    use crate::testing::{build_log, constants};

    #[test]
    fn test_next_run_ok() {
        let expected = format!(
            "Next Service Miles: {}\n\
            Next Service Date: {}",
            constants::NEXT_SERVICE.miles(),
            constants::NEXT_SERVICE.date()
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
