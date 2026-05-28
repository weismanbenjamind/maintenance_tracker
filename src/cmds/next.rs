use clap::Args;

use crate::containers::MaintenanceLog;
use crate::errors::{CmdsError, NextError};
use log::info;
use std::io::{self, Write};

#[derive(Clone, Debug, Args)]
#[command(about = "Get the next service event for a given maintenance item")]
pub struct Next {
    #[arg(help = "ID of service")]
    id: String,
}

impl Next {
    pub fn run(self, log: &MaintenanceLog) -> Result<(), CmdsError> {
        info!("Getting next service for id {}", self.id);
        let found = log
            .get(&self.id)
            .ok_or_else(|| CmdsError::IdNotFound((&self.id).into()))? // Borrow here because need below and don't want an accidental move here
            .next_service();

        info!("Writing next to stdout");
        let stdout = io::stdout();
        let mut buf = stdout.lock();
        writeln!(buf, "{found}").map_err(|e| NextError::FailedWrite((&self.id).into(), e))?; // Borrow here because need below and don't want an accidental move here

        info!("Finished getting next service for id {}", self.id);
        Ok(())
    }
}
