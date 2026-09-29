//! # Complete
//!
//! Command to mark a service as complete, update it's next service mileage/date,
//! and add a previous service to its service history.

use chrono::{Local, NaiveDate, TimeDelta};
use clap::Args;
use log::info;

use crate::{containers::MaintenanceLog, errors::CmdsError};

/// Houses args for the `Complete` command.
#[derive(Clone, Debug, Args)]
#[command(about = "Complete a service on specific day and mileage")]
pub(crate) struct Complete {
    #[arg(help = "ID of service")]
    id: String,

    #[arg(help = "Mileage on vehicle upon service completion")]
    mileage: u32,

    #[arg(short, long, default_value_t = Local::now().date_naive(), help = "Date on which service was completed")]
    date: NaiveDate,
}

impl Complete {
    /// Build a new Complete struct.
    /// Only used for testing.
    #[cfg(test)]
    pub(crate) fn new(id: &str, mileage: u32, date: NaiveDate) -> Self {
        Self {
            id: id.into(),
            mileage,
            date,
        }
    }

    /// Run the `Complete` command.
    pub(crate) fn run(self, log: &mut MaintenanceLog) -> Result<String, CmdsError> {
        info!(
            "Completing service with id '{}' at mileage {} on date {}.",
            self.id, self.mileage, self.date
        );

        let metadata = log.get_mut(&self.id)?;
        let next_service_miles = self.mileage + metadata.service_interval().miles();
        let next_service_date = self.date + TimeDelta::days(metadata.service_interval().days());

        let next_service_mut = metadata.next_service_mut();
        next_service_mut.set_miles(next_service_miles);
        next_service_mut.set_date(next_service_date);
        metadata.prev_services_mut().add(self.mileage, self.date);

        let msg = format!(
            "Marked {} as complete at {} miles on {}\n\
            Updated next service to {} miles or on {}",
            metadata.name(),
            self.mileage,
            self.date,
            next_service_miles,
            next_service_date
        );

        info!("Service logged as complete.");
        Ok(msg)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, TimeDelta};

    use super::*;
    use crate::{
        containers::ServiceEvent,
        testing::{
            build_log,
            constants::{ID, NAME, NEXT_SERVICE, PREVIOUS_SERVICE, SERVICE_INTERVAL},
        },
    };

    #[test]
    fn complete_new() {
        let miles = NEXT_SERVICE.miles();
        let date = NEXT_SERVICE.date();
        let found = Complete::new(ID, miles, date);

        assert_eq!(found.id, ID.to_string());
        assert_eq!(found.mileage, miles);
        assert_eq!(found.date, date);
    }

    #[test]
    fn complete_ok() {
        let mut log = build_log();

        let mileage = PREVIOUS_SERVICE.miles() + SERVICE_INTERVAL.miles() - 250;
        assert!(mileage > PREVIOUS_SERVICE.miles());

        let date = NEXT_SERVICE.date() - TimeDelta::days(30);
        assert!(date > PREVIOUS_SERVICE.date());

        let cmd = Complete {
            id: ID.into(),
            mileage,
            date,
        };

        let msg = cmd.run(&mut log).unwrap();

        let next_service_miles = mileage + SERVICE_INTERVAL.miles();
        let next_service_date = date + TimeDelta::days(SERVICE_INTERVAL.days());
        let previous_service = ServiceEvent::new(mileage, date);
        let updated_service = log.get(ID).unwrap();
        let next_service = updated_service.next_service();

        assert_eq!(next_service.miles(), next_service_miles);
        assert_eq!(next_service.date(), next_service_date);
        updated_service
            .prev_services()
            .service_events()
            .unwrap()
            .iter()
            .any(|s| *s == previous_service);

        let expected_msg = format!(
            "Marked {} as complete at {} miles on {}\n\
            Updated next service to {} miles or on {}",
            NAME, mileage, date, next_service_miles, next_service_date
        );

        assert_eq!(msg, expected_msg);
    }

    #[test]
    fn complete_err() {
        let mut log = build_log();
        let target_id = "some_id";
        assert!(!log.contains(target_id));
        let cmd = Complete {
            id: target_id.into(),
            mileage: 100000,
            date: NaiveDate::from_ymd_opt(2026, 6, 7).unwrap(),
        };
        match cmd.run(&mut log).unwrap_err() {
            CmdsError::IdNotFound(_) => (),
            _ => panic!("Expected CmdsError::IdNotFound"),
        }
    }
}
