use std::str::FromStr;

use crate::errors::InitError;
use chrono::NaiveDate;
use clap::Args;
use log::debug;

const YYYY_MM_DD: &str = "%Y-%m-%d";

#[derive(Clone, Debug, Args)]
#[command(about = "Initialize a service for tracking")]
pub struct Init {
    #[arg(long, help = "Name of service")]
    name: String,

    #[arg(long, help = "ID of service")]
    id: String,

    #[arg(long, help = "Miles interval service should be completed at")]
    miles_interval: u32,

    #[arg(long, help = "Month interval service should be completed at")]
    monthly_interval: u32,

    #[arg(
        long,
        help = "Mileage on vehcile when next service should be completed"
    )]
    next_service_miles: u32,

    #[arg(long, help = "Date which next service should be completed")]
    next_service_date: NaiveDate,

    #[arg(long, short, long, help = "Notes about service")]
    notes: Option<Vec<String>>,

    #[arg(
        long,
        short,
        long,
        help = "Previous service. Should be in format 'miles:YYYY-MM-DD' where miles is a positive integer"
    )]
    previous_services: Vec<PreviousService>,
}

#[derive(Clone, Copy, Debug, Args)]
pub struct PreviousService {
    #[arg(short, long, help = "Mileage on vehicle at previous service event")]
    miles: u32,

    #[arg(short, long, help = "Date of previous service event")]
    date: NaiveDate,
}

impl FromStr for PreviousService {
    type Err = InitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split(";");

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
