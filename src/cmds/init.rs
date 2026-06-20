use std::str::FromStr;

use crate::containers::{MaintenanceLog, ServiceMetdata};
use crate::errors::{CmdsError, InitError};
use chrono::NaiveDate;
use clap::Args;
use log::{debug, info};

const YYYY_MM_DD: &str = "%Y-%m-%d";
const DELIMITER: &str = ";";

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

#[derive(Clone, Copy, Debug)]
pub(crate) struct PreviousService {
    miles: u32,

    date: NaiveDate,
}

impl PreviousService {
    pub(crate) fn miles(&self) -> u32 {
        self.miles
    }

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

#[derive(Clone, Copy, Debug, Args)]
pub(crate) struct ServiceInterval {
    #[arg(help = "Miles interval service should be completed at")]
    miles_interval: u32,

    #[arg(help = "Month interval service should be completed at")]
    monthly_interval: u32,
}

impl ServiceInterval {
    pub(crate) fn miles_interval(&self) -> u32 {
        self.miles_interval
    }

    pub(crate) fn monthly_interval(&self) -> u32 {
        self.monthly_interval
    }
}

#[derive(Clone, Copy, Debug, Args)]
pub(crate) struct NextService {
    #[arg(help = "Mileage on vehcile when next service should be completed")]
    next_service_miles: u32,

    #[arg(help = "Date which next service should be completed")]
    next_service_date: NaiveDate,
}

impl NextService {
    pub(crate) fn next_service_miles(&self) -> u32 {
        self.next_service_miles
    }

    pub(crate) fn next_service_date(&self) -> NaiveDate {
        self.next_service_date
    }
}
