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
    pub fn new(id: &str) -> Self {
        Self { id: id.into() }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn run(self, log: &MaintenanceLog) -> Result<(), CmdsError> {
        info!("Getting next service for id {}", self.id);
        let found = log
            .get(&self.id)
            .ok_or_else(|| NextError::IdNotFound(self.id().into()))?
            .next_service();

        info!("Writing next to stdout");
        let stdout = io::stdout();
        let mut buf = stdout.lock();
        writeln!(buf, "{found}").map_err(|e| NextError::FailedWrite(self.id().into(), e))?;

        info!("Finished getting next service for id {}", self.id);
        Ok(())
    }
}
