//! # Init module
//!
//! Housing functionality for initializing a new service

use std::str::FromStr;

use crate::containers::{MaintenanceLog, ServiceMetdata};
use crate::errors::{CmdsError, InitError};
use chrono::NaiveDate;
use clap::Args;
use log::{debug, info};

const YYYY_MM_DD: &str = "%Y-%m-%d";
const DELIMITER: &str = ";";

/// Struct housing args for initializing a new service.
#[derive(Clone, Debug, Args)]
#[command(about = "Initialize a service for tracking")]
pub(crate) struct Init {
    #[arg(help = "Name of service")]
    name: String,

    #[arg(help = "ID of service")]
    id: String,

    #[command(flatten)]
    service_interval: ServiceInterval,

    #[command(flatten)]
    next_service: NextService,

    #[arg(long, short, long, help = "Notes about service")]
    notes: Option<Vec<String>>,

    #[arg(
        long,
        short,
        long,
        help = "Previous services. Should be in format 'miles;YYYY-MM-DD' where miles is a positive integer"
    )]
    previous_services: Option<Vec<PreviousService>>,
}

impl Init {
    /// Create a new Init struct.
    /// Only used for testing.
    #[cfg(test)]
    pub(crate) fn new(
        name: &str,
        id: &str,
        service_interval: ServiceInterval,
        next_service: NextService,
        notes: Option<Vec<String>>,
        previous_services: Option<Vec<PreviousService>>,
    ) -> Self {
        Self {
            name: name.into(),
            id: id.into(),
            service_interval,
            next_service,
            notes,
            previous_services,
        }
    }

    /// Run service initialization logic.
    pub(crate) fn run(self, log: &mut MaintenanceLog) -> Result<String, CmdsError> {
        info!(
            "Initializing service with id {} and name {}.",
            self.id, self.name
        );

        if log.contains(&self.id) {
            return Err(CmdsError::IdExists(self.id, self.name));
        }

        let previous_services = self.previous_services.map(|prev_services| {
            prev_services
                .into_iter()
                .map(|service| service.into())
                .collect()
        });

        let update = ServiceMetdata::new(
            &self.name,
            self.service_interval.into(),
            self.next_service.into(),
            previous_services,
            self.notes,
        );

        let msg = format!(
            "Initialized the following service:\n\nId: {}\n{update}",
            self.id
        );

        // Don't need to check the return type here because we already check that the id is not present at
        // the start of this function
        log.insert(&self.id, update);
        info!(
            "Successfully initialized service with id {} and name {}.",
            self.id, self.name
        );

        Ok(msg)
    }
}

/// Struct to house previous service info.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PreviousService {
    miles: u32,
    date: NaiveDate,
}

impl PreviousService {
    /// Build a new PreviousService struct.
    /// Only used for testing.
    #[cfg(test)]
    pub(crate) fn new(miles: u32, date: NaiveDate) -> Self {
        Self { miles, date }
    }

    /// Get a copy of the miles attribute.
    pub(crate) fn miles(&self) -> u32 {
        self.miles
    }

    /// Get a copy of the date attribute.
    pub(crate) fn date(self) -> NaiveDate {
        self.date
    }
}

impl FromStr for PreviousService {
    type Err = InitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split(DELIMITER);

        let (miles, date) = match (split.next(), split.next(), split.next()) {
            (Some(miles), Some(date), None) => {
                debug!("When parsing previous service found miles {miles} and date {date}.");
                (miles, date)
            }
            _ => Err(InitError::FailedPreviousServiceParse)?,
        };

        Ok(Self {
            miles: miles.parse::<u32>()?,
            date: NaiveDate::parse_from_str(date, YYYY_MM_DD)?,
        })
    }
}

/// Houses service interval args.
#[derive(Clone, Copy, Debug, PartialEq, Args)]
pub(crate) struct ServiceInterval {
    #[arg(help = "Miles interval service should be completed at")]
    miles_interval: u32,

    #[arg(help = "Month interval service should be completed at")]
    monthly_interval: u32,
}

impl ServiceInterval {
    /// Build a new ServiceInterval struct.
    /// Only used for testing.
    #[cfg(test)]
    pub(crate) fn new(miles_interval: u32, monthly_interval: u32) -> Self {
        Self {
            miles_interval,
            monthly_interval,
        }
    }

    /// Get a copy of the miles interval.
    pub(crate) fn miles_interval(&self) -> u32 {
        self.miles_interval
    }

    /// Get a copy of the monthly interval.
    pub(crate) fn monthly_interval(&self) -> u32 {
        self.monthly_interval
    }
}

/// Houses args for next service.
#[derive(Clone, Copy, Debug, Args)]
pub(crate) struct NextService {
    #[arg(help = "Mileage on vehcile when next service should be completed")]
    next_service_miles: u32,

    #[arg(help = "Date which next service should be completed")]
    next_service_date: NaiveDate,
}

impl NextService {
    /// Build a NextService struct.
    /// Only used for testing.
    #[cfg(test)]
    pub(crate) fn new(miles: u32, date: NaiveDate) -> Self {
        Self {
            next_service_miles: miles,
            next_service_date: date,
        }
    }

    /// Get a clone of the next service miles.
    pub(crate) fn next_service_miles(&self) -> u32 {
        self.next_service_miles
    }

    /// Get a clone of the next service date.
    pub(crate) fn next_service_date(&self) -> NaiveDate {
        self.next_service_date
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::build_log;
    use crate::testing::constants::ID;

    #[test]
    fn init_new() {
        let name = "name";
        let id = "id";
        let service_interval = ServiceInterval {
            miles_interval: 4000,
            monthly_interval: 5,
        };
        let next_service = NextService {
            next_service_miles: 3000,
            next_service_date: NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
        };
        let notes = None;
        let previous_services = None;

        let args = Init::new(
            name,
            id,
            service_interval,
            next_service,
            notes.clone(),
            previous_services.clone(),
        );

        assert_eq!(&args.name, name);
        assert_eq!(&args.id, id);
        assert_eq!(args.service_interval, service_interval);
        assert_eq!(args.notes, notes);
        assert_eq!(args.previous_services, previous_services);
    }

    #[test]
    fn init_run_ok() {
        let new_id = "new_id";
        let mut log = build_log();
        assert!(!log.contains(new_id));

        let name = "Service";
        let service_interval = ServiceInterval {
            miles_interval: 4000,
            monthly_interval: 5,
        };
        let next_service = NextService {
            next_service_miles: 70000,
            next_service_date: NaiveDate::from_ymd_opt(2026, 6, 7).unwrap(),
        };
        let notes = Some(
            vec!["note_1", "note_2"]
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<String>>(),
        );
        let previous_services = Some(vec![PreviousService {
            miles: 66000,
            date: NaiveDate::from_ymd_opt(2026, 1, 7).unwrap(),
        }]);

        let cmd = Init {
            name: name.into(),
            id: new_id.into(),
            service_interval,
            next_service,
            notes: notes.clone(),
            previous_services: previous_services.clone(),
        };

        let _msg = cmd.run(&mut log).unwrap();

        let found = log.get(new_id).unwrap();
        assert_eq!(found.name(), name);

        let found_service_interval = found.service_interval();
        assert_eq!(
            found_service_interval.miles(),
            service_interval.miles_interval
        );
        assert_eq!(
            found_service_interval.days(),
            (service_interval.monthly_interval as f64 / 12.0 * 365.0).floor() as i64
        );

        let found_next_service = found.next_service();
        assert_eq!(found_next_service.miles(), next_service.next_service_miles);
        assert_eq!(found_next_service.date(), next_service.next_service_date);

        assert_eq!(found.notes().try_get(), notes.as_deref());

        let found_prev_services = found.prev_services().service_events().unwrap();
        assert_eq!(found_prev_services.len(), 1);
        let found_prev_service = found_prev_services[0];

        let expected_prev_service = previous_services.unwrap()[0];

        assert_eq!(
            (found_prev_service.miles(), found_prev_service.date()),
            (expected_prev_service.miles(), expected_prev_service.date())
        );
    }

    #[test]
    fn init_run_err() {
        let cmd = Init {
            name: "Service".into(),
            id: ID.into(),
            service_interval: ServiceInterval {
                miles_interval: 4000,
                monthly_interval: 5,
            },
            next_service: NextService {
                next_service_miles: 70000,
                next_service_date: NaiveDate::from_ymd_opt(2026, 6, 7).unwrap(),
            },
            notes: None,
            previous_services: None,
        };

        match cmd.run(&mut build_log()).unwrap_err() {
            CmdsError::IdExists(_, _) => (),
            _ => panic!("Expected CmdsError::IdExists"),
        }
    }

    #[test]
    fn init_previous_service() {
        let miles = 1000;
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let found = PreviousService::new(miles, date);
        assert_eq!(found.miles(), miles);
        assert_eq!(found.date(), date);
    }

    #[test]
    fn init_previous_service_from_str_ok() {
        let to_parse = "100000;2026-06-02";
        let found = PreviousService::from_str(to_parse).unwrap();
        assert_eq!(
            found,
            PreviousService {
                miles: 100000,
                date: NaiveDate::from_ymd_opt(2026, 6, 2).unwrap()
            }
        )
    }

    #[test]
    fn init_previous_service_from_str_bad_miles_err() {
        let to_parse = "100s00;2026-06-02";
        match PreviousService::from_str(to_parse).unwrap_err() {
            InitError::InvalidMilesFormat(_) => (),
            _ => panic!("Expected InitError::InvalidMilesFormat"),
        }
    }

    #[test]
    fn init_previous_service_from_str_bad_date_err() {
        let to_parse = "100000;2026-0a-02";
        match PreviousService::from_str(to_parse).unwrap_err() {
            InitError::InvalidDateFormat(_) => (),
            _ => panic!("Expected InitError::InvalidDateFormat"),
        }
    }

    #[test]
    fn init_previous_service_from_str_bad_delimiter() {
        let to_parse = "100000:2026-06-02";
        match PreviousService::from_str(to_parse).unwrap_err() {
            InitError::FailedPreviousServiceParse => (),
            _ => panic!("InitError::FailedPreviousServiceParse"),
        }
    }

    #[test]
    fn init_previous_service_from_str_multi_delimiter() {
        let to_parse = "100000;;2026-06-02";
        match PreviousService::from_str(to_parse).unwrap_err() {
            InitError::FailedPreviousServiceParse => (),
            _ => panic!("InitError::FailedPreviousServiceParse"),
        }
    }

    #[test]
    fn init_service_interval() {
        let miles = 1000;
        let monthly_interval = 5;
        let found = ServiceInterval::new(miles, monthly_interval);
        assert_eq!(found.miles_interval(), miles);
        assert_eq!(found.monthly_interval(), monthly_interval);
    }

    #[test]
    fn init_next_service() {
        let miles = 1000;
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let found = NextService::new(miles, date);
        assert_eq!(found.next_service_miles(), miles);
        assert_eq!(found.next_service_date, date);
    }
}
