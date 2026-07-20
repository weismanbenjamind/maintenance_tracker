//! # Delete
//!
//! Houses command for deleting a service with a given ID

use clap::Args;
use log::info;

use crate::containers::MaintenanceLog;
use crate::errors::{CmdsError, IdNotFoundError};

/// Args for delete command.
#[derive(Clone, Debug, Args)]
#[command(about = "Delete data for a specific service")]
pub(crate) struct Delete {
    #[arg(help = "Id of service")]
    id: String,
}

impl Delete {
    // TODO - test this
    /// Create a new Delete struct.
    /// Only used for testing.
    #[cfg(test)]
    pub(crate) fn new(id: &str) -> Self {
        Self { id: id.into() }
    }

    /// Delete a service with a given ID.
    pub(crate) fn run(self, log: &mut MaintenanceLog) -> Result<String, CmdsError> {
        info!("Attempting to delete service with id {}", self.id);
        let deleted = log
            .remove(&self.id)
            .ok_or(IdNotFoundError::IdNotFound(self.id))?;

        let msg = format!("Deleted service {}", deleted.name());

        info!("Service removed");
        Ok(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::build_log;
    use crate::testing::constants::ID;

    #[test]
    fn delete_ok() {
        let mut log = build_log();
        let cmd = Delete { id: ID.into() };
        assert!(log.contains(ID));
        let found = cmd.run(&mut log).unwrap();
        assert!(found.to_lowercase().contains("deleted service"));
        assert!(!log.contains(ID));
    }

    #[test]
    fn delete_err() {
        let to_delete = "to_delete";
        let mut log = build_log();
        assert!(!log.contains(to_delete));

        let cmd = Delete {
            id: to_delete.into(),
        };

        match cmd.run(&mut log).unwrap_err() {
            CmdsError::IdNotFound(_) => (),
            _ => panic!("Expected CmdsError::IdNotFound"),
        }
    }
}
