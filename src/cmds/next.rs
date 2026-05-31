use clap::Args;

use crate::containers::MaintenanceLog;
use crate::errors::{CmdsError, NextError};
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
        let found = match log.get(&self.id) {
            Some(metadata) => metadata.next_service(),
            None => return Err(CmdsError::IdNotFound(self.id)),
        };

        info!("Writing next to stdout.");
        let stdout = io::stdout();
        let mut buf = stdout.lock();

        if let Err(e) = writeln!(buf, "{found}") {
            return Err(NextError::FailedWrite(self.id, e).into());
        }

        info!("Finished getting next service for id {}.", self.id);
        Ok(())
    }
}
