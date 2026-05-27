// Using a unit (static) struct hre to follow all command patters
use crate::errors::ListError;
use crate::{containers::MaintenanceLog, errors::CmdsError};
use clap::Args;
use log::info;
use std::io::{self, Write};

#[derive(Clone, Copy, Debug, Args)]
#[command(about = "List all services and their ids")]
pub struct List;

impl List {
    // Need to return a cmds error because in main we return a maintenance log error
    // Errors can only bubble up one level ?
    // So return the level right below MaintenanceLogError - which is Cmds Error
    pub fn run(self, log: &MaintenanceLog) -> Result<(), CmdsError> {
        info!("Writing maintenance log ids to stdout");
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        log.ids()
            // Map error to ListError::FailedList to convert to a MaintenanceLogError
            .try_for_each(|id| writeln!(handle, "{id}").map_err(ListError::FailedList))?;
        info!("Successfully wrote maintenance log ids to stdout");
        Ok(())
    }
}
