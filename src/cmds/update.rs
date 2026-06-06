mod notes;
mod service;

use notes::{self as notes_cmds};
use service::{self as service_cmds};

use crate::containers::{MaintenanceLog, ServiceMetdata};
use crate::errors::{CmdsError, UpdateError};
use clap::{Args, Subcommand};
use log::info;

#[derive(Clone, Debug, Args)]
#[command(about = "Update a service")]
pub(crate) struct Update {
    #[arg(help = "ID of service to update")]
    id: String,

    #[command(subcommand)]
    cmd: Cmd,
}

impl Update {
    // TODO - need a command to update the next service in here
    pub(crate) fn run(self, log: &mut MaintenanceLog) -> Result<(), CmdsError> {
        info!("Updating maintenance log.");

        let mut metadata = match log.remove(&self.id) {
            Some(metadata) => metadata,
            None => return Err(CmdsError::IdNotFound(self.id)),
        };

        // If want to update the id need to remove the current metadata and insert at the new id
        // Another option is to always remove then for the id cmd just update the self.id attribute to be the update
        // At the end just put the metadata back with the proper id
        let mut id_to_insert: Option<String> = None;

        match self.cmd {
            Cmd::Name(args) => metadata.set_name(&args.name),
            Cmd::Id(args) => id_to_insert = Some(args.id), // Set the id variable so when we insert back into the map we insert with the new id
            Cmd::MilesInterval(args) => metadata.set_service_interval_miles(args.miles),
            Cmd::MonthInterval(args) => metadata.set_service_interval_months(args.months),
            Cmd::Notes(update_notes_cmd) => update_notes_cmd.run(&mut metadata)?,
            Cmd::Service(update_service_cmd) => update_service_cmd.run(&mut metadata)?,
        }

        // Don't need to check the option return since when we remove above we are sure this key does not exist
        // We also check during the update id cmd that they new id does not exist
        log.insert(&id_to_insert.unwrap_or(self.id), metadata);

        Ok(())
    }
}

#[derive(Clone, Debug, Subcommand)]
enum Cmd {
    Name(UpdateName),
    Id(UpdateId),
    MilesInterval(UpdateMilesInterval),
    MonthInterval(UpdateMonthInterval),
    Notes(UpdateNotes),
    Service(UpdateService),
}

#[derive(Clone, Debug, Args)]
#[command(about = "Update name")]
struct UpdateName {
    #[arg(help = "New name of service")]
    name: String,
}

#[derive(Clone, Debug, Args)]
#[command(about = "Update id")]
struct UpdateId {
    #[arg(help = "New id of service")]
    id: String,
}

#[derive(Clone, Copy, Debug, Args)]
#[command(about = "Update miles interval the service should be completed at")]
struct UpdateMilesInterval {
    #[arg(help = "New miles interval")]
    miles: u32,
}

#[derive(Clone, Copy, Debug, Args)]
#[command(about = "Update monthly interval the service should be completed at")]
struct UpdateMonthInterval {
    #[arg(help = "New monthly interval")]
    months: u32,
}

#[derive(Clone, Debug, Args)]
#[command(about = "Update notes")]
struct UpdateNotes {
    #[command(subcommand)]
    cmd: UpdateNotesCmd,
}

impl UpdateNotes {
    fn run(self, metadata: &mut ServiceMetdata) -> Result<(), UpdateError> {
        match self.cmd {
            UpdateNotesCmd::Append(args) => {
                metadata.exetend_notes(args.into_notes());
                Ok(())
            }
            UpdateNotesCmd::Replace(args) => {
                let (index, contents) = args.into_parts();
                metadata.replace_note(index, &contents)?;
                Ok(())
            }
            UpdateNotesCmd::Insert(args) => {
                let (index, contents) = args.into_parts();
                metadata.insert_note(index, &contents)?;
                Ok(())
            }
            UpdateNotesCmd::Remove(args) => {
                metadata.remove_note(args.index())?;
                Ok(())
            }
            UpdateNotesCmd::Clear(_) => {
                metadata.clear_notes();
                Ok(())
            }
        }
    }
}

#[derive(Clone, Debug, Subcommand)]
enum UpdateNotesCmd {
    Append(notes_cmds::Append),
    Replace(notes_cmds::Replace),
    Insert(notes_cmds::Insert),
    Remove(notes_cmds::Remove),
    Clear(notes_cmds::Clear),
}

#[derive(Clone, Copy, Debug, Args)]
#[command(about = "Update a previous service")]
struct UpdateService {
    #[command(subcommand)]
    cmd: UpdateServiceCmd,
}

impl UpdateService {
    fn run(self, metadata: &mut ServiceMetdata) -> Result<(), UpdateError> {
        match self.cmd {
            UpdateServiceCmd::Append(args) => {
                metadata.add_service_event(args.miles(), args.date());
                Ok(())
            }
            UpdateServiceCmd::Replace(args) => {
                metadata.replace_service_event(
                    args.curr_miles(),
                    args.curr_date(),
                    args.new_miles(),
                    args.new_date(),
                )?;
                Ok(())
            }
            UpdateServiceCmd::Remove(args) => {
                metadata.remove_service_event(args.miles(), args.date())?;
                Ok(())
            }
            UpdateServiceCmd::Clear(_) => {
                metadata.clear_previous_services();
                Ok(())
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Subcommand)]
enum UpdateServiceCmd {
    Append(service_cmds::Append),
    Replace(service_cmds::Replace),
    Remove(service_cmds::Remove),
    Clear(service_cmds::Clear),
}
