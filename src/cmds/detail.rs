use clap::Args;
use log::info;
use std::io::{self, Write};

use crate::containers::MaintenanceLog;
use crate::errors::{CmdsError, DetailError};

#[derive(Clone, Debug, Args)]
#[command(about = "Show details about a specific service")]
pub struct Detail {
    #[arg(help = "ID of service")]
    id: String,
}

impl Detail {
    pub fn new(id: &str) -> Self {
        Self { id: id.into() }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn run(self, log: &MaintenanceLog) -> Result<(), CmdsError> {
        info!(
            "Getting details for maintenance log item with id {}",
            self.id
        );

        match log.get(&self.id) {
            Some(found) => {
                let stdout = io::stdout();
                let mut buf = stdout.lock();
                writeln!(buf, "{found}").map_err(|e| DetailError::FailedWrite(self.id, e))?
            }
            None => return Err(CmdsError::IdNotFound(self.id)),
        };

        info!("Successfully got details for maintenance log item");

        Ok(())
    }
}
