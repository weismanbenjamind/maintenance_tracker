use crate::errors::CmdsError;
use crate::{containers::MaintenanceLog, errors::CompleteError};
use chrono::{Local, NaiveDate, TimeDelta};
use clap::Args;
use log::info;
use std::io::{self, Write};

#[derive(Clone, Debug, Args)]
#[command(about = "Complete a service on specific day and mileage")]
pub struct Complete {
    #[arg(short, long, help = "ID of service")]
    id: String,

    #[arg(short, long, help = "Mileage on vehicle upon service completion")]
    mileage: u32,

    #[arg(short, long, default_value_t = Local::now().date_naive(), help = "Date on which service was completed")]
    date: NaiveDate,
}

impl Complete {
    pub fn run(self, log: &mut MaintenanceLog) -> Result<(), CmdsError> {
        info!(
            "Completing service with id '{}' at mileage {} on date {}",
            self.id, self.mileage, self.date
        );

        let metadata = log
            .get_mut(&self.id)
            .ok_or_else(|| CmdsError::IdNotFound((&self.id).into()))?; // Have to borrow and convert to a string since might move here and need below

        let next_service_miles = self.mileage + metadata.service_interval().miles;
        let next_service_date = self.date + TimeDelta::days(metadata.service_interval().days());

        metadata.set_next_service(next_service_miles, next_service_date);
        metadata.add_service_event(self.mileage, self.date);

        let stdout = io::stdout();
        let mut buf = stdout.lock();

        writeln!(
            buf,
            "Marked {} as complete at {} miles on {}.\n\
            Updated next service to {} miles or on {}.",
            metadata.name, self.mileage, self.date, next_service_miles, next_service_date
        )
        .map_err(|e| CompleteError::FailedWrite(self.id, e))?;

        info!("Service logged as complete");
        Ok(())
    }
}
