use super::notes::UpdateNotesCmd;
use super::service::UpdateServiceCmd;

use crate::containers::{MaintenanceLog, ServiceMetdata};
use crate::errors::{CmdsError, UpdateError};
use chrono::NaiveDate;
use clap::{Args, Subcommand};
use log::{debug, info};

#[derive(Clone, Debug, Args)]
#[command(about = "Update a service")]
pub(crate) struct Update {
    #[arg(help = "ID of service to update")]
    id: String,

    #[command(subcommand)]
    cmd: Cmd,
}

impl Update {
    pub(crate) fn run(self, log: &mut MaintenanceLog) -> Result<(), CmdsError> {
        info!("Updating maintenance log.");

        let mut metadata = match log.remove(&self.id) {
            Some(metadata) => metadata,
            None => return Err(CmdsError::new_id_not_found(&self.id)),
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
            Cmd::NextService(update_next_service) => update_next_service.run(&mut metadata)?,
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
    NextService(UpdateNextService),
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

#[derive(Clone, Copy, Debug, Args)]
#[command(about = "Update next service")]
#[group(required = true, multiple = true)]
struct UpdateNextService {
    #[arg(short, long, help = "Target next service miles")]
    miles: Option<u32>,

    #[arg(short, long, help = "Target next service date")]
    date: Option<NaiveDate>,
}

impl UpdateNextService {
    fn run(&self, metadata: &mut ServiceMetdata) -> Result<(), UpdateError> {
        match (self.miles, self.date) {
            (None, None) => return Err(UpdateError::UpdateNextServiceArgs),
            (Some(_), None) | (None, Some(_)) | (Some(_), Some(_)) => {
                debug!(
                    "Updating next service with miles: {:?} and date {:?}",
                    self.miles, self.date
                );
                metadata.next_service_mut().update(self.miles, self.date)
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Args)]
#[command(about = "Update notes")]
struct UpdateNotes {
    #[command(subcommand)]
    cmd: UpdateNotesCmd,
}

impl UpdateNotes {
    fn run(self, metadata: &mut ServiceMetdata) -> Result<(), UpdateError> {
        let notes = metadata.notes_mut();
        match self.cmd {
            UpdateNotesCmd::Append(args) => {
                notes.extend(args.into_notes());
                Ok(())
            }
            UpdateNotesCmd::Replace(args) => {
                let (idx, contents) = args.into_parts();
                notes.replace(idx, &contents)?;
                Ok(())
            }
            UpdateNotesCmd::Insert(args) => {
                let (idx, contents) = args.into_parts();
                notes.insert(idx, &contents)?;
                Ok(())
            }
            UpdateNotesCmd::Remove(args) => {
                notes.remove(args.index())?;
                Ok(())
            }
            UpdateNotesCmd::Clear(_) => {
                notes.clear();
                Ok(())
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Args)]
#[command(about = "Update a previous service")]
struct UpdateService {
    #[command(subcommand)]
    cmd: UpdateServiceCmd,
}

impl UpdateService {
    fn run(self, metadata: &mut ServiceMetdata) -> Result<(), UpdateError> {
        let prev_services = metadata.prev_services_mut();
        match self.cmd {
            UpdateServiceCmd::Append(args) => {
                prev_services.add(args.miles(), args.date());
                Ok(())
            }
            UpdateServiceCmd::Replace(args) => {
                prev_services.replace(
                    args.curr_miles(),
                    args.curr_date(),
                    args.new_miles(),
                    args.new_date(),
                )?;
                Ok(())
            }
            UpdateServiceCmd::Remove(args) => {
                prev_services.remove(args.miles(), args.date())?;
                Ok(())
            }
            UpdateServiceCmd::Clear(_) => {
                prev_services.clear();
                Ok(())
            }
        }
    }
}
