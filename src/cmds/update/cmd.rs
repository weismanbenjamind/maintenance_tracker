//! # Command
//!
//! Houses commands, utilities, and functions for updating previous services

use super::notes::UpdateNotesCmd;
use super::prev_services::UpdatePreviousServicesCmd;

use crate::containers::{MaintenanceLog, ServiceMetdata};
use crate::errors::{CmdsError, UpdateError};
use chrono::NaiveDate;
use clap::{Args, Subcommand};
use log::{debug, info};
use std::fmt::Write;

/// Args for updating a service.
#[derive(Clone, Debug, Args)]
#[command(about = "Update a service")]
pub(crate) struct Update {
    #[arg(help = "ID of service to update")]
    id: String,

    #[command(subcommand)]
    cmd: Cmd,
}

impl Update {
    /// Run the update service command
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

/// Enum to house all potential commands and their args for a service update
#[derive(Clone, Debug, Subcommand)]
enum Cmd {
    Name(UpdateName),
    Id(UpdateId),
    ServiceInterval(UpdateServiceInterval),
    NextService(UpdateNextService),
    Notes(UpdateNotes),
    PreviousServices(UpdatePreviousServices),
}

/// Args for updating a service name
#[derive(Clone, Debug, Args)]
#[command(about = "Update name")]
struct UpdateName {
    #[arg(help = "New name of service")]
    name: String,
}

/// Args for updating a service id
#[derive(Clone, Debug, Args)]
#[command(about = "Update id")]
struct UpdateId {
    #[arg(help = "New id of service")]
    id: String,
}

/// Args for updating a service interval
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
    /// Run the update service interval command
    fn run(self, metadata: &mut ServiceMetdata) -> Result<String, UpdateError> {
        // Clap should enforce this constraint - but still check
        if self.miles.is_none() && self.months.is_none() {
            return Err(UpdateError::InvalidServiceIntervalUpdateArgs);
        };

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

/// Get the message the user will see when a service interval is updates
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

/// Args for updating the next service
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
    /// Run the update next service command
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

/// Get the message a user will see when updating the next service
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

/// Args for updating notes
#[derive(Clone, Debug, Args)]
#[command(about = "Update notes")]
struct UpdateNotes {
    #[command(subcommand)]
    cmd: UpdateNotesCmd,
}

impl UpdateNotes {
    /// Run the update notes command
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
                Ok(format!("Successfully inserted note at index {idx}"))
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

/// Args for updating a previous service
#[derive(Clone, Copy, Debug, Args)]
#[command(about = "Update previous services")]
struct UpdatePreviousServices {
    #[command(subcommand)]
    cmd: UpdatePreviousServicesCmd,
}

impl UpdatePreviousServices {
    /// Run the update previous services command
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

/// Get the message the user will see when a service get replaced
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

/// Get the message a user will see when a service gets removed
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

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;

    use super::*;
    use crate::cmds::update::{notes, prev_services};
    use crate::testing::constants::{
        ID, NAME, NEXT_SERVICE, NOTE, PREVIOUS_SERVICE, SERVICE_INTERVAL,
    };
    use crate::testing::{build_log, build_metadata};

    #[test]
    fn upate_cmd_run_prev_services() {
        let update_args = Update {
            id: ID.into(),
            cmd: Cmd::PreviousServices(UpdatePreviousServices {
                cmd: UpdatePreviousServicesCmd::Clear(prev_services::Clear),
            }),
        };
        let mut log = build_log();
        assert!(log.get(ID).unwrap().prev_services().len().is_some());

        let found = update_args.run(&mut log).unwrap();
        assert_eq!(found, "Successfully cleared previous services");
        assert!(log.get(ID).unwrap().prev_services().len().is_none());
    }

    #[test]
    fn upate_cmd_run_notes() {
        let update_args = Update {
            id: ID.into(),
            cmd: Cmd::Notes(UpdateNotes {
                cmd: UpdateNotesCmd::Clear(notes::Clear),
            }),
        };
        let mut log = build_log();
        assert!(log.get(ID).unwrap().notes().len().is_some());

        let found = update_args.run(&mut log).unwrap();
        assert_eq!(found, "Successfully cleared notes");
        assert!(log.get(ID).unwrap().notes().len().is_none());
    }

    #[test]
    fn upate_cmd_run_next_service() {
        let miles = 1000;
        assert_ne!(NEXT_SERVICE.miles(), miles);

        let update_args = Update {
            id: ID.into(),
            cmd: Cmd::NextService(UpdateNextService {
                miles: Some(miles),
                date: None,
            }),
        };
        let mut log = build_log();

        let found = update_args.run(&mut log).unwrap();
        assert!(found.to_lowercase().contains("set next service miles"));
        assert_eq!(log.get(ID).unwrap().next_service().miles(), miles);
    }

    #[test]
    fn upate_cmd_run_service_interval() {
        let miles = 1000;
        assert_ne!(SERVICE_INTERVAL.miles(), miles);

        let update_args = Update {
            id: ID.into(),
            cmd: Cmd::ServiceInterval(UpdateServiceInterval {
                miles: Some(miles),
                months: None,
            }),
        };
        let mut log = build_log();

        let found = update_args.run(&mut log).unwrap();
        assert!(
            found
                .to_lowercase()
                .contains("updated service interval miles")
        );
        assert_eq!(log.get(ID).unwrap().service_interval().miles(), miles);
    }

    #[test]
    fn upate_cmd_run_id() {
        let new_id = "new_id";
        assert_ne!(new_id, ID);

        let update_args = Update {
            id: ID.into(),
            cmd: Cmd::Id(UpdateId { id: new_id.into() }),
        };
        let mut log = build_log();

        let found = update_args.run(&mut log).unwrap();
        assert_eq!(found, format!("Updated id to {new_id}"));
        assert!(log.get(new_id).is_ok());
        assert!(log.get(ID).is_err());
    }

    #[test]
    fn upate_cmd_run_name() {
        let new_name = "new_name";
        assert_ne!(new_name, NAME);

        let update_args = Update {
            id: ID.into(),
            cmd: Cmd::Name(UpdateName {
                name: new_name.into(),
            }),
        };
        let mut log = build_log();

        let found = update_args.run(&mut log).unwrap();
        assert_eq!(found, format!("Updated name to {new_name}"));
        assert_eq!(log.get(ID).unwrap().name(), new_name);
    }

    #[test]
    fn upate_cmd_run_id_not_found() {
        let id = "non_found";
        assert_ne!(id, ID);

        let update_args = Update {
            id: id.into(),
            cmd: Cmd::Name(UpdateName {
                name: "some_name".into(),
            }),
        };

        let found = update_args.run(&mut build_log()).unwrap_err();
        assert!(matches!(found, CmdsError::IdNotFound(_)))
    }

    #[test]
    fn update_service_interval_run_miles_and_months() {
        let mut metadata = build_metadata();
        let service_interval = metadata.service_interval();

        let miles = service_interval.miles() - 100;
        let months = service_interval.months() - 1;
        let args = UpdateServiceInterval {
            miles: Some(miles),
            months: Some(months),
        };

        let found = args.run(&mut metadata).unwrap().to_lowercase();
        assert!(found.contains("service interval miles"));
        assert!(found.contains("service interval months"));

        let service_interval = metadata.service_interval();
        assert_eq!(service_interval.miles(), miles);
        assert_eq!(service_interval.months(), months);
    }

    #[test]
    fn update_service_interval_run_months() {
        let mut metadata = build_metadata();
        let service_interval = metadata.service_interval();

        let original_miles = service_interval.miles();
        let months = service_interval.months() - 1;
        let args = UpdateServiceInterval {
            miles: None,
            months: Some(months),
        };

        let found = args.run(&mut metadata).unwrap().to_lowercase();
        assert!(found.contains("service interval months"));

        let service_interval = metadata.service_interval();
        assert_eq!(service_interval.miles(), original_miles);
        assert_eq!(service_interval.months(), months);
    }

    #[test]
    fn update_service_interval_run_miles() {
        let mut metadata = build_metadata();
        let service_interval = metadata.service_interval();

        let miles = service_interval.miles() - 100;
        let original_months = service_interval.months();
        let args = UpdateServiceInterval {
            miles: Some(miles),
            months: None,
        };

        let found = args.run(&mut metadata).unwrap().to_lowercase();
        assert!(found.contains("service interval miles"));

        let service_interval = metadata.service_interval();
        assert_eq!(service_interval.miles(), miles);
        assert_eq!(service_interval.months(), original_months);
    }

    #[test]
    fn update_service_interval_err() {
        let mut metadata = build_metadata();
        let args = UpdateServiceInterval {
            miles: None,
            months: None,
        };

        let found = args.run(&mut metadata).unwrap_err();
        assert!(matches!(
            found,
            UpdateError::InvalidServiceIntervalUpdateArgs
        ));
    }

    #[test]
    fn get_updated_service_interval_msg_miles_and_months() {
        let miles = 4000;
        let months = 5;
        let found = get_updated_service_interval_msg(Some(miles), Some(months)).unwrap();
        assert_eq!(
            found,
            format!(
                "Updated service interval miles to {miles} miles and service interval months to {months} months"
            )
        )
    }

    #[test]
    fn get_updated_service_interval_msg_date() {
        let months = 5;
        let found = get_updated_service_interval_msg(None, Some(months)).unwrap();
        assert_eq!(
            found,
            format!("Updated service interval months to {months} months")
        )
    }

    #[test]
    fn get_updated_service_interval_msg_miles() {
        let miles = 4000;
        let found = get_updated_service_interval_msg(Some(miles), None).unwrap();
        assert_eq!(
            found,
            format!("Updated service interval miles to {miles} miles")
        )
    }

    #[test]
    fn get_updated_service_interval_msg_err() {
        let found = get_updated_service_interval_msg(None, None).unwrap_err();
        assert!(matches!(
            found,
            UpdateError::InvalidServiceIntervalUpdateArgs
        ))
    }

    #[test]
    fn update_next_service_run_miles_and_date() {
        let miles = NEXT_SERVICE.miles() + 100;
        let date = NEXT_SERVICE.date() + TimeDelta::days(100);
        let args = UpdateNextService {
            miles: Some(miles),
            date: Some(date),
        };
        let mut metdata = build_metadata();

        let found = args.run(&mut metdata).unwrap();
        assert!(found.to_lowercase().contains("next service miles"));
        assert!(found.to_lowercase().contains("next service date"));
        let next_service = metdata.next_service();
        assert_eq!(next_service.miles(), miles);
        assert_eq!(next_service.date(), date);
    }

    #[test]
    fn update_next_service_run_date() {
        let date = NEXT_SERVICE.date() + TimeDelta::days(100);
        let args = UpdateNextService {
            miles: None,
            date: Some(date),
        };
        let mut metdata = build_metadata();

        let found = args.run(&mut metdata).unwrap();
        assert!(found.to_lowercase().contains("set next service date"));
        assert_eq!(metdata.next_service().date(), date)
    }

    #[test]
    fn update_next_service_run_miles() {
        let miles = NEXT_SERVICE.miles() + 100;
        let args = UpdateNextService {
            miles: Some(miles),
            date: None,
        };
        let mut metdata = build_metadata();

        let found = args.run(&mut metdata).unwrap();
        assert!(found.to_lowercase().contains("set next service"));
        assert_eq!(metdata.next_service().miles(), miles)
    }

    #[test]
    fn update_next_service_run_err() {
        let args = UpdateNextService {
            miles: None,
            date: None,
        };
        let found = args.run(&mut build_metadata()).unwrap_err();
        assert!(matches!(found, UpdateError::UpdateNextServiceArgs))
    }

    #[test]
    fn get_next_service_update_msg_miles_and_date() {
        let miles = 1000;
        let date = NaiveDate::from_ymd_opt(2026, 6, 1);
        let found = get_next_service_update_msg(Some(miles), date).unwrap();
        assert_eq!(
            found,
            format!(
                "Set next service miles to {miles} and next service date to {}",
                date.unwrap()
            )
        )
    }

    #[test]
    fn get_next_service_update_msg_date() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 1);
        let found = get_next_service_update_msg(None, date).unwrap();
        assert_eq!(found, format!("Set next service date to {}", date.unwrap()))
    }

    #[test]
    fn get_next_service_update_msg_miles() {
        let miles = 1000;
        let found = get_next_service_update_msg(Some(miles), None).unwrap();
        assert_eq!(found, format!("Set next service miles to {miles}"))
    }

    #[test]
    fn get_next_service_update_msg_err() {
        assert!(matches!(
            get_next_service_update_msg(None, None).unwrap_err(),
            UpdateError::UpdateNextServiceArgs
        ))
    }

    #[test]
    fn cmd_update_notes_clear() {
        let mut metadata = build_metadata();
        assert_eq!(metadata.notes().len().unwrap(), 1);

        let clear_args = notes::Clear;
        let clear_cmd = UpdateNotesCmd::Clear(clear_args);
        let clear_notes_args = UpdateNotes { cmd: clear_cmd };

        let result = clear_notes_args.run(&mut metadata).unwrap();
        assert!(result.to_lowercase().contains("successfully cleared"));
        assert!(metadata.notes().try_get().is_none())
    }

    #[test]
    fn cmd_update_notes_remove_one() {
        let mut metadata = build_metadata();
        assert_eq!(metadata.notes().len().unwrap(), 1);
        metadata.notes_mut().insert(0, "note").unwrap();
        assert_eq!(metadata.notes().len().unwrap(), 2);

        let remove_args = notes::Remove::new(1);
        let remove_cmd = UpdateNotesCmd::Remove(remove_args);
        let update_notes_args = UpdateNotes { cmd: remove_cmd };

        let result = update_notes_args.run(&mut metadata).unwrap();
        assert!(result.to_lowercase().contains("successfully removed"));
        assert_eq!(metadata.notes().len().unwrap(), 1);
    }

    #[test]
    fn cmd_update_notes_remove_all() {
        let mut metadata = build_metadata();
        assert_eq!(metadata.notes().len().unwrap(), 1);

        let remove_args = notes::Remove::new(0);
        let remove_cmd = UpdateNotesCmd::Remove(remove_args);
        let update_notes_args = UpdateNotes { cmd: remove_cmd };

        let result = update_notes_args.run(&mut metadata).unwrap();
        assert!(result.to_lowercase().contains("successfully removed"));
        let notes = metadata.notes();
        assert!(notes.try_get().is_none());
    }

    #[test]
    fn cmd_update_notes_insert() {
        let mut metadata = build_metadata();
        assert_eq!(metadata.notes().len().unwrap(), 1);

        let to_insert = "note_1";
        let insert_args = notes::Insert::new(0, to_insert);
        let insert_cmd = UpdateNotesCmd::Insert(insert_args);
        let update_notes_args = UpdateNotes { cmd: insert_cmd };

        let result = update_notes_args.run(&mut metadata).unwrap();
        assert!(result.to_lowercase().contains("successfully inserted"));
        let notes = metadata.notes();
        assert_eq!(notes.len().unwrap(), 2);
        assert_eq!(notes.try_get().unwrap()[0], to_insert);
    }

    #[test]
    fn cmd_update_notes_replace() {
        let mut metadata = build_metadata();
        assert_eq!(metadata.notes().len().unwrap(), 1);

        let replacement = "note";
        assert_ne!(replacement, NOTE);
        let replace_args = notes::Replace::new(0, replacement);
        let replace_cmd = UpdateNotesCmd::Replace(replace_args);
        let update_notes_args = UpdateNotes { cmd: replace_cmd };

        let result = update_notes_args.run(&mut metadata).unwrap();
        assert!(result.to_lowercase().contains("successfully replaced note"));
        let notes = metadata.notes();
        assert_eq!(notes.len().unwrap(), 1);
        assert_eq!(notes.try_get().unwrap()[0], replacement);
    }

    #[test]
    fn cmd_update_notes_append() {
        let mut metadata = build_metadata();
        assert_eq!(metadata.notes().len().unwrap(), 1);

        let to_append = ["note_1", "note_2"]
            .iter()
            .map(|note| note.to_string())
            .collect::<Vec<String>>();
        let append_args = notes::Append::new(&to_append);
        let args = UpdateNotesCmd::Append(append_args);
        let args = UpdateNotes { cmd: args };

        let result = args.run(&mut metadata).unwrap();
        assert!(result.to_lowercase().contains("successfully appended"));
        assert_eq!(metadata.notes().len().unwrap(), 3);
    }

    #[test]
    fn cmd_update_prev_services_clear() {
        let mut metadata = build_metadata();

        let args = UpdatePreviousServices {
            cmd: UpdatePreviousServicesCmd::Clear(prev_services::Clear),
        };

        let found = args.run(&mut metadata).unwrap();
        assert!(found.to_lowercase().contains("cleared previous services"));
        assert!(metadata.prev_services().service_events().is_none());
    }

    #[test]
    fn cmd_update_prev_services_run_remove() {
        let mut metadata = build_metadata();

        let remove_args = prev_services::Remove::new(Some(PREVIOUS_SERVICE.miles()), None);
        let args = UpdatePreviousServices {
            cmd: UpdatePreviousServicesCmd::Remove(remove_args),
        };

        let found = args.run(&mut metadata).unwrap();
        assert!(found.to_lowercase().contains("removed service with miles"));
        assert!(metadata.prev_services().service_events().is_none());
    }

    #[test]
    fn cmd_update_prev_services_run_replace() {
        let mut metadata = build_metadata();
        let prev_service_miles = PREVIOUS_SERVICE.miles();
        let prev_service_date = PREVIOUS_SERVICE.date();

        let new_miles = prev_service_miles + 5000;
        let new_date = prev_service_date + TimeDelta::days(120);

        let replace_args = prev_services::Replace::new(
            Some(prev_service_miles),
            Some(prev_service_date),
            Some(new_miles),
            Some(new_date),
        );
        let args = UpdatePreviousServices {
            cmd: UpdatePreviousServicesCmd::Replace(replace_args),
        };

        let result = args.run(&mut metadata).unwrap().to_lowercase();
        assert!(result.contains("with miles"));
        assert!(result.contains("and date"));
        assert!(result.contains("with new miles"));
        assert!(result.contains("and new date"));

        let prev_services = metadata.prev_services().service_events().unwrap();
        assert!(prev_services.len() == 1);

        let prev_service = prev_services[0];
        assert_eq!(prev_service.miles(), new_miles);
        assert_eq!(prev_service.date(), new_date);
    }

    #[test]
    fn cmd_update_prev_services_run_append() {
        let mut metadata = build_metadata();
        assert!(metadata.prev_services().len().unwrap() == 1);

        let append_args =
            prev_services::Append::new(100000, NaiveDate::from_ymd_opt(2026, 7, 1).unwrap());
        let args = UpdatePreviousServices {
            cmd: UpdatePreviousServicesCmd::Append(append_args),
        };

        let result = args.run(&mut metadata).unwrap();
        assert!(result.to_lowercase().contains("added previous service"));
        assert!(metadata.prev_services().len().unwrap() == 2);
    }

    #[test]
    fn cmd_get_replaced_service_msg_new_miles_and_new_date() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 3);
        let found = get_replaced_service_msg(Some(70000), date, Some(70000), date).unwrap();
        assert!(found.contains("with new miles"));
        assert!(found.contains("and new date"));
    }

    #[test]
    fn cmd_get_replaced_service_msg_new_date() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 3);
        let found = get_replaced_service_msg(None, date, None, date)
            .unwrap()
            .to_lowercase();
        assert!(found.contains("with new date"));
        assert!(!found.contains("with new miles"));
    }

    #[test]
    fn cmd_get_replaced_service_msg_new_miles() {
        let found = get_replaced_service_msg(Some(1000), None, Some(1000), None)
            .unwrap()
            .to_lowercase();
        assert!(found.contains("with new miles"));
        assert!(!found.contains("with new date"));
    }

    #[test]
    fn cmd_get_replaced_service_msg_new_args_err() {
        let _found = get_replaced_service_msg(Some(1000), None, None, None).unwrap_err();
        assert!(matches!(UpdateError::InvalidServiceUpdateArgs, _found));
    }

    #[test]
    fn cmd_get_replaced_service_msg_curr_miles_and_curr_date() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 3);
        let found = get_replaced_service_msg(Some(75000), date, Some(1000), date)
            .unwrap()
            .to_lowercase();

        assert!(found.contains("successfully updated service with miles"));
        assert!(found.contains(" and new date "))
    }

    #[test]
    fn cmd_get_replaced_service_msg_curr_date() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 3);
        let found = get_replaced_service_msg(None, date, None, date)
            .unwrap()
            .to_lowercase();

        assert!(found.contains("successfully updated service with date"));
        assert!(!found.contains(" miles "));
    }

    #[test]
    fn cmd_get_replaced_service_msg_curr_miles() {
        let found = get_replaced_service_msg(Some(75000), None, Some(1000), None)
            .unwrap()
            .to_lowercase();

        assert!(found.contains("successfully updated service with miles"));
        assert!(!found.contains(" date "));
    }

    #[test]
    fn cmd_get_replaced_service_msg_curr_args_err() {
        let _found =
            get_replaced_service_msg(None, None, Some(1000), NaiveDate::from_ymd_opt(2026, 6, 3))
                .unwrap_err();
        assert!(matches!(UpdateError::InvalidCurrentServiceIds, _found));
    }

    #[test]
    fn cmd_get_removed_service_msg_miles_and_date() {
        let found = get_removed_service_msg(Some(1000), NaiveDate::from_ymd_opt(2026, 6, 3))
            .unwrap()
            .to_lowercase();
        assert!(found.contains("removed service with miles"));
        assert!(found.contains("and date"))
    }

    #[test]
    fn cmd_get_removed_service_msg_date() {
        let found = get_removed_service_msg(None, NaiveDate::from_ymd_opt(2026, 6, 3)).unwrap();
        assert!(found.to_lowercase().contains("removed service with date"));
    }

    #[test]
    fn cmd_get_removed_service_msg_miles() {
        let found = get_removed_service_msg(Some(1000), None).unwrap();
        assert!(found.to_lowercase().contains("removed service with miles"));
    }

    #[test]
    fn cmd_get_removed_service_msg_err() {
        assert!(matches!(
            get_removed_service_msg(None, None).unwrap_err(),
            UpdateError::InvalidCurrentServiceIds
        ));
    }
}
