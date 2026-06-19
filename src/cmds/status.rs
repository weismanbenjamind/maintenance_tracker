use chrono::{Local, NaiveDate};
use clap::Args;
use log::{debug, info};
use std::fmt::Write;
use std::io::{self, Write as WriteIO};

use crate::containers::MaintenanceLog;
use crate::errors::CmdsError;

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
            Some(id) => vec![log.get(&id)?],
            None => log.metadata_sorted(),
        };

        debug!("Writing status for metadata {:?}", metadata);

        let mut buf = String::new();

        metadata.iter().for_each(|m| {
            let next_service = m.next_service();
            let status_result = StatusResult {
                name: m.name(),
                curr_miles: self.curr_miles,
                next_service_miles: next_service.miles(),
                date: self.today,
                next_service_date: next_service.date(),
            };
            // Writing to a string can't fail
            _ = writeln!(buf, "{status_result}\n");
        });

        let stdout = io::stdout();
        let mut stdout_buf = stdout.lock();
        writeln!(stdout_buf, "{}", buf.trim_end())?;

        info!("Status operation complete");

        Ok(())
    }
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
        writeln!(f, "Next service (miles): {} Miles", self.next_service_miles)?;
        writeln!(f, "Current mileage: {} Miles", self.curr_miles)?;
        writeln!(
            f,
            "Miles until next service (Next Service Miles - Current Miles): {} Miles",
            self.miles_diff()
        )?;
        writeln!(f, "Next service date: {}", self.next_service_date)?;
        writeln!(f, "Today: {}", self.date)?;
        write!(
            f,
            "Days until next service (Next Service Date - Today): {} Days",
            self.days_diff()
        )
    }
}
