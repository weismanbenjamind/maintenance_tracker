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
    #[test]
    fn delete_ok() {}

    #[test]
    fn delete_err() {}
}
