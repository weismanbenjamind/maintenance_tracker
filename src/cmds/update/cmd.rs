use super::notes::UpdateNotesCmd;
use super::prev_services::UpdatePreviousServicesCmd;

use crate::containers::{MaintenanceLog, ServiceMetdata};
use crate::errors::{CmdsError, UpdateError};
use chrono::NaiveDate;
use clap::{Args, Subcommand};
use log::{debug, info};
use std::fmt::Write;

#[derive(Clone, Debug, Args)]
#[command(about = "Update a service")]
pub(crate) struct Update {
    #[arg(help = "ID of service to update")]
    id: String,

    #[command(subcommand)]
    cmd: Cmd,
}

impl Update {
    pub(crate) fn run(self, log: &mut MaintenanceLog) -> Result<String, CmdsError> {
        info!("Updating maintenance log.");

        let mut metadata = match log.remove(&self.id) {
            Some(metadata) => metadata,
            None => return Err(CmdsError::new_id_not_found(&self.id)),
        };

        // If want to update the id need to remove the current metadata and insert at the new id
        // Another option is to always remove then for the id cmd just update the self.id attribute to be the update
        // At the end just put the metadata back with the proper id
        let mut id_to_insert: Option<String> = None;

        let mut buf = String::new();

        // In all write! calls below writing to string which cannot fail
        match self.cmd {
            Cmd::Name(args) => {
                metadata.set_name(&args.name);
                _ = write!(buf, "Updated name to {}", args.name);
            }
            Cmd::ServiceInterval(args) => {
                let msg = args.run(&mut metadata)?;
                _ = write!(buf, "{msg}")
            }
            Cmd::NextService(update_next_service) => {
                let msg = update_next_service.run(&mut metadata)?;
                _ = write!(buf, "{msg}")
            }
            Cmd::Notes(update_notes_cmd) => {
                let msg = update_notes_cmd.run(&mut metadata)?;
                _ = write!(buf, "{msg}")
            }
            Cmd::PreviousServices(update_service_cmd) => {
                let msg = update_service_cmd.run(&mut metadata)?;
                _ = write!(buf, "{msg}")
            }
            // Set the id variable so when we insert back into the map we insert with the new id.
            // Will inform user of this update below
            Cmd::Id(args) => id_to_insert = Some(args.id),
        }

        // Don't need to check the option return since when we remove above we are sure this key does not exist
        // We also check during the update id cmd that they new id does not exist
        // log.insert can't fail so okay with informing user of update before it happens
        if let Some(new_id) = &id_to_insert {
            // Writing to string which cannot fail
            _ = write!(buf, "Updated id to {new_id}");
        }
        log.insert(&id_to_insert.unwrap_or(self.id), metadata);

        Ok(buf)
    }
}

#[derive(Clone, Debug, Subcommand)]
enum Cmd {
    Name(UpdateName),
    Id(UpdateId),
    ServiceInterval(UpdateServiceInterval),
    NextService(UpdateNextService),
    Notes(UpdateNotes),
    PreviousServices(UpdatePreviousServices),
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
#[command(about = "Update service interval")]
#[group(required = true, multiple = true)]
struct UpdateServiceInterval {
    #[arg(short, long, help = "New miles interval")]
    miles: Option<u32>,

    #[arg(short = 'n', long, help = "New monthly interval")]
    months: Option<u32>,
}

impl UpdateServiceInterval {
    fn run(self, metadata: &mut ServiceMetdata) -> Result<String, UpdateError> {
        let service_interval = metadata.service_interval_mut();
        let (miles, months) = (self.miles, self.months);

        if let Some(miles) = miles {
            service_interval.set_miles(miles);
        }

        if let Some(months) = months {
            service_interval.set_months(months);
        }

        get_updated_service_interval_msg(miles, months)
    }
}

fn get_updated_service_interval_msg(
    miles: Option<u32>,
    months: Option<u32>,
) -> Result<String, UpdateError> {
    let msg = match (miles, months) {
        (Some(miles), None) => format!("Updated service interval miles to {miles} miles"),
        (None, Some(months)) => format!("Updated service interval months to {months} months"),
        (Some(miles), Some(months)) => format!(
            "Updated service interval miles to {miles} miles and service interval months to {months} months"
        ),
        (None, None) => return Err(UpdateError::InvalidServiceIntervalUpdateArgs),
    };
    Ok(msg)
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
    fn run(&self, metadata: &mut ServiceMetdata) -> Result<String, UpdateError> {
        match (self.miles, self.date) {
            (None, None) => Err(UpdateError::UpdateNextServiceArgs),
            (Some(_), None) | (None, Some(_)) | (Some(_), Some(_)) => {
                debug!(
                    "Updating next service with miles: {:?} and date {:?}",
                    self.miles, self.date
                );
                metadata.next_service_mut().update(self.miles, self.date);
                // Below should never error in practice because we are in the match arm where the (None, None) case cannot occur
                let msg = get_next_service_update_msg(self.miles, self.date)?;
                Ok(msg)
            }
        }
    }
}

fn get_next_service_update_msg(
    miles: Option<u32>,
    date: Option<NaiveDate>,
) -> Result<String, UpdateError> {
    match (miles, date) {
        (Some(miles), None) => Ok(format!("Set next service miles to {miles}")),
        (None, Some(date)) => Ok(format!("Set next service date to {date}")),
        (Some(miles), Some(date)) => Ok(format!(
            "Set next service miles to {miles} and next service date to {date}"
        )),
        (None, None) => Err(UpdateError::UpdateNextServiceArgs),
    }
}

#[derive(Clone, Debug, Args)]
#[command(about = "Update notes")]
struct UpdateNotes {
    #[command(subcommand)]
    cmd: UpdateNotesCmd,
}

impl UpdateNotes {
    fn run(self, metadata: &mut ServiceMetdata) -> Result<String, UpdateError> {
        let notes = metadata.notes_mut();
        match self.cmd {
            UpdateNotesCmd::Append(args) => {
                let to_append = args.into_notes();
                let num_notes = to_append.len() as u32;
                notes.extend(to_append);
                Ok(format!("Successfully appended {num_notes} note(s)"))
            }
            UpdateNotesCmd::Replace(args) => {
                let (idx, contents) = args.into_parts();
                notes.replace(idx, &contents)?;
                Ok(format!("Successfully replaced note note at index {idx}"))
            }
            UpdateNotesCmd::Insert(args) => {
                let (idx, contents) = args.into_parts();
                notes.insert(idx, &contents)?;
                Ok(format!("Successfully replaced note at index {idx}"))
            }
            UpdateNotesCmd::Remove(args) => {
                let idx = args.index();
                notes.remove(idx)?;
                Ok(format!("Successfully removed note at index {idx}"))
            }
            UpdateNotesCmd::Clear(_) => {
                notes.clear();
                Ok("Successfully cleared notes".to_string())
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Args)]
#[command(about = "Update previous services")]
struct UpdatePreviousServices {
    #[command(subcommand)]
    cmd: UpdatePreviousServicesCmd,
}

impl UpdatePreviousServices {
    fn run(self, metadata: &mut ServiceMetdata) -> Result<String, UpdateError> {
        let prev_services = metadata.prev_services_mut();
        match self.cmd {
            UpdatePreviousServicesCmd::Append(args) => {
                let (miles, date) = (args.miles(), args.date());
                prev_services.add(miles, date);
                Ok(format!(
                    "Successfully added previous service with miles {miles} and date {date}"
                ))
            }
            UpdatePreviousServicesCmd::Replace(args) => {
                let (curr_miles, curr_date, new_miles, new_date) = args.into_parts();
                prev_services.replace(curr_miles, curr_date, new_miles, new_date)?;
                let msg = get_replaced_service_msg(curr_miles, curr_date, new_miles, new_date)?;
                Ok(msg)
            }
            UpdatePreviousServicesCmd::Remove(args) => {
                let (miles, date) = args.into_parts();
                prev_services.remove(miles, date)?;
                let msg = get_removed_service_msg(miles, date)?;
                Ok(msg)
            }
            UpdatePreviousServicesCmd::Clear(_) => {
                prev_services.clear();
                Ok("Successfully cleared previous services".to_string())
            }
        }
    }
}

fn get_replaced_service_msg(
    curr_miles: Option<u32>,
    curr_date: Option<NaiveDate>,
    new_miles: Option<u32>,
    new_date: Option<NaiveDate>,
) -> Result<String, UpdateError> {
    let mut buf = String::new();

    // Write to string can't fail so unwrap below
    match (curr_miles, curr_date) {
        (Some(curr_miles), None) => {
            write!(buf, "Successfully updated service with miles {curr_miles}").unwrap()
        }
        (None, Some(curr_date)) => {
            write!(buf, "Successfully updated service with date {curr_date}").unwrap()
        }
        (Some(curr_miles), Some(curr_date)) => write!(
            buf,
            "Successfully updated service with miles {curr_miles} and date {curr_date}"
        )
        .unwrap(),
        // In practice this error should never get hit - clap should take care of input validation
        (None, None) => return Err(UpdateError::InvalidCurrentServiceIds),
    };

    // Write to string can't fail so unwrap below
    match (new_miles, new_date) {
        (Some(new_miles), None) => write!(buf, " with new miles {new_miles}").unwrap(),
        (None, Some(new_date)) => write!(buf, " with new date {new_date}").unwrap(),
        (Some(new_miles), Some(new_date)) => {
            write!(buf, " with new miles {new_miles} and new date {new_date}").unwrap()
        }
        // In practice this error should never get hit - clap should take care of input validation
        (None, None) => return Err(UpdateError::InvalidServiceUpdateArgs),
    }

    Ok(buf)
}

fn get_removed_service_msg(
    miles: Option<u32>,
    date: Option<NaiveDate>,
) -> Result<String, UpdateError> {
    let msg = match (miles, date) {
        (Some(miles), None) => format!("Removed service with miles {miles}"),
        (None, Some(date)) => format!("Removed service with date {date}"),
        (Some(miles), Some(date)) => format!("Removed service with miles {miles} and date {date}"),
        // In practice this error should never get hit - clap should take care of input validation
        (None, None) => return Err(UpdateError::InvalidCurrentServiceIds),
    };

    Ok(msg)
}
