use chrono::{Local, NaiveDate};
use clap::Args;
use log::{debug, info};
use std::io::{self, StdoutLock, Write as WriteIO};

use crate::containers::{MaintenanceLog, ServiceMetdata};
use crate::errors::{CmdsError, StatusError};

#[derive(Clone, Debug, Args)]
#[command(about = "Get the miles/date difference for a specific service or all services")]
pub(crate) struct Status {
    #[arg(help = "Current miles on vehicle")]
    curr_miles: u32,

    #[arg(
        short,
        long,
        help = "Optional id to get status for. If omitted status will be grabbed for all services"
    )]
    id: Option<String>,

    #[arg(
        short,
        long,
        help = "Override 'today' when calculating the difference between 'today' and the date the service is due",
        default_value_t = Local::now().date_naive(),
    )]
    today: NaiveDate,
}

impl Status {
    pub(crate) fn run(self, log: &MaintenanceLog) -> Result<(), CmdsError> {
        info!("Starting status operation");

        let metadata = match self.id {
            Some(id) => {
                vec![log.get(&id)?]
            }
            None => log.metadata().collect::<Vec<&ServiceMetdata>>(),
        };

        debug!("Writing status for metadata {:?}", metadata);

        let stdout = io::stdout();
        let mut buf = stdout.lock();

        writeln!(buf).map_err(StatusError::FailedStdoutWrite)?;
        metadata.iter().try_for_each(|m| {
            let next_service = m.next_service();
            let status_result = StatusResult {
                name: m.name(),
                curr_miles: self.curr_miles,
                next_service_miles: next_service.miles(),
                date: self.today,
                next_service_date: next_service.date(),
            };
            write_to_buf(&mut buf, status_result)
        })?;

        info!("Status operation complete");

        Ok(())
    }
}

fn write_to_buf<T: std::fmt::Display>(
    buf: &mut StdoutLock,
    contents: T,
) -> Result<(), StatusError> {
    writeln!(buf, "{contents}\n").map_err(StatusError::FailedStdoutWrite)
}

#[derive(Clone, Copy, Debug)]
struct StatusResult<'a> {
    name: &'a str,
    curr_miles: u32,
    next_service_miles: u32,
    date: NaiveDate,
    next_service_date: NaiveDate,
}

impl<'a> StatusResult<'a> {
    fn miles_diff(&self) -> i64 {
        self.next_service_miles as i64 - self.curr_miles as i64
    }

    fn days_diff(&self) -> i64 {
        (self.next_service_date - self.date).num_days()
    }
}

impl<'a> std::fmt::Display for StatusResult<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Name: {}", self.name)?;
        writeln!(f, "Next Service (miles): {}", self.next_service_miles)?;
        writeln!(f, "Current Miles: {}", self.curr_miles)?;
        writeln!(
            f,
            "Next Service Miles - Current Miles: {} Miles",
            self.miles_diff()
        )?;
        writeln!(f, "Next Service Date: {}", self.next_service_date)?;
        writeln!(f, "Today: {}", self.date)?;
        write!(f, "Next Service Date - Today: {} Days", self.days_diff())
    }
}
