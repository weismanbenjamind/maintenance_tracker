use std::str::FromStr;

use crate::containers;
use crate::errors::{CmdsError, InitError};
use chrono::NaiveDate;
use clap::Args;
use log::{debug, info};

const YYYY_MM_DD: &str = "%Y-%m-%d";
const DELIMITER: &str = ";";

#[derive(Clone, Debug, Args)]
#[command(about = "Initialize a service for tracking")]
pub struct Init {
    #[arg(long, help = "Name of service")]
    name: String,

    #[arg(long, help = "ID of service")]
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
        help = "Previous service. Should be in format 'miles;YYYY-MM-DD' where miles is a positive integer"
    )]
    previous_services: Option<Vec<PreviousService>>,
}

impl Init {
    pub fn new(
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn service_interval(&self) -> ServiceInterval {
        self.service_interval
    }

    pub fn next_service(&self) -> NextService {
        self.next_service
    }

    pub fn notes(&self) -> Option<&[String]> {
        self.notes.as_deref()
    }

    pub fn previous_services(&self) -> Option<&[PreviousService]> {
        self.previous_services.as_deref()
    }

    pub fn run(self, log: &mut containers::MaintenanceLog) -> Result<(), CmdsError> {
        info!(
            "Initializing service with id {} and name {}",
            self.id, self.name
        );

        if log.contains(&self.id) {
            return Err(InitError::IdExists(self.id, self.name).into());
        }

        let previous_services = self.previous_services.map(|prev_services| {
            prev_services
                .into_iter()
                .map(|service| service.into())
                .collect()
        });

        let update = containers::ServiceMetdata::new(
            &self.name,
            self.service_interval.into(),
            self.next_service.into(),
            previous_services,
            self.notes,
        );

        // Don't need to check the return type here because we already check that the id is not present at
        // the start of this function
        log.insert(&self.id, update);
        info!(
            "Successfully initialized service with id {} and name {}",
            self.id, self.name
        );

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Args)]
pub struct PreviousService {
    #[arg(short, long, help = "Mileage on vehicle at previous service event")]
    miles: u32,

    #[arg(short, long, help = "Date of previous service event")]
    date: NaiveDate,
}

impl PreviousService {
    pub fn new(miles: u32, date: NaiveDate) -> Self {
        Self { miles, date }
    }

    pub fn miles(&self) -> u32 {
        self.miles
    }

    pub fn date(self) -> NaiveDate {
        self.date
    }
}

impl FromStr for PreviousService {
    type Err = InitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split(DELIMITER);

        let (miles, date) = match (split.next(), split.next(), split.next()) {
            (Some(miles), Some(date), None) => {
                debug!("When parsing previous service found miles {miles} and date {date}");
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
pub struct ServiceInterval {
    #[arg(long, help = "Miles interval service should be completed at")]
    miles_interval: u32,

    #[arg(long, help = "Month interval service should be completed at")]
    monthly_interval: u32,
}

impl ServiceInterval {
    pub fn new(miles_interval: u32, monthly_interval: u32) -> Self {
        Self {
            miles_interval,
            monthly_interval,
        }
    }

    pub fn miles_interval(&self) -> u32 {
        self.miles_interval
    }

    pub fn monthly_interval(&self) -> u32 {
        self.monthly_interval
    }
}

#[derive(Clone, Copy, Debug, Args)]
pub struct NextService {
    #[arg(
        long,
        help = "Mileage on vehcile when next service should be completed"
    )]
    next_service_miles: u32,

    #[arg(long, help = "Date which next service should be completed")]
    next_service_date: NaiveDate,
}

impl NextService {
    pub fn new(next_service_miles: u32, next_service_date: NaiveDate) -> Self {
        Self {
            next_service_miles,
            next_service_date,
        }
    }

    pub fn next_service_miles(&self) -> u32 {
        self.next_service_miles
    }

    pub fn next_service_date(&self) -> NaiveDate {
        self.next_service_date
    }
}
