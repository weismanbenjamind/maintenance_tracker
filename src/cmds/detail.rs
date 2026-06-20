use clap::Args;
use log::info;

use crate::containers::MaintenanceLog;
use crate::errors::CmdsError;

#[derive(Clone, Debug, Args)]
#[command(about = "Show details about a specific service")]
pub(crate) struct Detail {
    #[arg(help = "ID of service")]
    id: String,
}

impl Detail {
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
