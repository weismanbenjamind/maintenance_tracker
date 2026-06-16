use clap::Args;
use log::info;
use std::io::{self, Write};

use crate::containers::MaintenanceLog;
use crate::errors::CmdsError;

#[derive(Clone, Debug, Args)]
#[command(about = "Show details about a specific service")]
pub(crate) struct Detail {
    #[arg(help = "ID of service")]
    id: String,
}

impl Detail {
    pub(crate) fn run(self, log: &MaintenanceLog) -> Result<(), CmdsError> {
        info!(
            "Getting details for maintenance log item with id {}.",
            self.id
        );

        let metadata = log.get(&self.id)?;
        let stdout = io::stdout();
        let mut buf = stdout.lock();
        writeln!(buf, "{metadata}")?;

        info!("Successfully got details for maintenance log item.");

        Ok(())
    }
}
