use clap::Args;

use crate::containers::MaintenanceLog;
use crate::errors::CmdsError;
use log::info;
use std::io::{self, Write};

#[derive(Clone, Debug, Args)]
#[command(about = "Get the next service event for a given maintenance item")]
pub(crate) struct Next {
    #[arg(help = "ID of service")]
    id: String,
}

impl Next {
    pub(crate) fn run(self, log: &MaintenanceLog) -> Result<(), CmdsError> {
        info!("Getting next service for id {}.", self.id);
        let found = log.get(&self.id)?;

        info!("Writing next to stdout.");
        let stdout = io::stdout();
        let mut buf = stdout.lock();
        writeln!(buf, "{found}")?;

        info!("Finished getting next service for id {}.", self.id);
        Ok(())
    }
}
