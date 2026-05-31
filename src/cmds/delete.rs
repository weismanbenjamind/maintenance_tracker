use clap::Args;
use log::info;

use crate::containers::MaintenanceLog;
use crate::errors::CmdsError;

#[derive(Clone, Debug, Args)]
#[command(about = "Delete data for a specific service")]
pub struct Delete {
    #[arg(help = "Id of service")]
    id: String,
}

impl Delete {
    pub fn run(self, log: &mut MaintenanceLog) -> Result<(), CmdsError> {
        info!("Attempting to delete service with id {}", self.id);
        log.remove(&self.id).ok_or(CmdsError::IdNotFound(self.id))?;
        info!("Service removed");
        Ok(())
    }
}
