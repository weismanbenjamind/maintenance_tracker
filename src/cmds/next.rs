use clap::Args;

use crate::containers::MaintenanceLog;
use crate::errors::CmdsError;
use log::info;

#[derive(Clone, Debug, Args)]
#[command(about = "Get the next service event for a given maintenance item")]
pub(crate) struct Next {
    #[arg(help = "ID of service")]
    id: String,
}

impl Next {
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
