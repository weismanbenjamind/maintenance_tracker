// Using a unit (static) struct hre to follow all command patters
use crate::containers::MaintenanceLog;
use crate::errors::CmdsError;
use clap::Args;
use log::info;
use std::fmt::Write;

#[derive(Clone, Copy, Debug, Args)]
#[command(about = "List all services and their ids")]
pub(crate) struct List;

impl List {
    // Need to return a cmds error because in main we return a maintenance log error
    // Errors can only bubble up one level ?
    // So return the level right below MaintenanceLogError - which is Cmds Error
    // Also this function technically cannot fail but return a result to be consistent with other .run() methods
    // ^ Makes error handling easier
    pub(crate) fn run(self, log: &MaintenanceLog) -> Result<String, CmdsError> {
        info!("Writing maintenance log ids to stdout.");

        // Keep ids sorted alphabetically - have to collect
        let mut ids: Vec<&String> = log.ids().collect();
        ids.sort();

        let mut buf = String::new();
        // Writing to a string can't fail so unwrap
        ids.iter().for_each(|id| writeln!(buf, "{id}").unwrap());

        info!("Successfully wrote maintenance log ids to stdout.");
        Ok(buf.trim_end().into())
    }
}
