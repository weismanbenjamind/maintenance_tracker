//! # Status
//!
//! Houses command for getting status of a single ID or multiple IDs

use chrono::{Local, NaiveDate};
use clap::Args;
use log::{debug, info};
use std::fmt::Write;

use crate::containers::MaintenanceLog;
use crate::errors::CmdsError;

/// Arguments for Status command.
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
    /// Run the status command given the arguments housed in the struct.
    pub(crate) fn run(self, log: &MaintenanceLog) -> Result<String, CmdsError> {
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

        info!("Status operation complete");

        Ok(buf.trim_end().into())
    }
}

/// View struct which houses the output of a status calculation.
/// Allows for easy formatting.
#[derive(Clone, Copy, Debug)]
struct StatusResult<'a> {
    name: &'a str,
    curr_miles: u32,
    next_service_miles: u32,
    date: NaiveDate,
    next_service_date: NaiveDate,
}

impl<'a> StatusResult<'a> {
    /// Get the miles differences between the next service miles
    /// and current miles on the vehicle.
    fn miles_diff(&self) -> i64 {
        self.next_service_miles as i64 - self.curr_miles as i64
    }

    /// Get the difference in days between the next service date
    /// and the date passed to the StatusResult struct.
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

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;

    use super::*;
    use crate::testing::constants::{ID, NAME, NEXT_SERVICE};
    use crate::testing::{build_log, build_metadata};

    #[test]
    fn status_run_multi_id() {
        let mut log = build_log();
        let metadata = build_metadata();
        let new_id = "new_id";
        assert!(!log.contains(new_id));
        log.insert(new_id, metadata);

        let miles_diff: i32 = 3000;
        let miles = NEXT_SERVICE.miles() - miles_diff as u32;

        let days_diff = 30;
        let today = NEXT_SERVICE.date() - TimeDelta::days(days_diff);

        let cmd = Status {
            curr_miles: miles,
            id: None,
            today,
        };

        let found = cmd.run(&log).unwrap();

        let mut buf = String::new();
        output_string_to_buf(&mut buf, miles, today, miles_diff, days_diff);
        _ = writeln!(buf);
        _ = writeln!(buf);
        output_string_to_buf(&mut buf, miles, today, miles_diff, days_diff);

        assert_eq!(found, buf);
    }

    #[test]
    fn status_run_single_id() {
        let miles_diff: i32 = 3000;
        let miles = NEXT_SERVICE.miles() - miles_diff as u32;

        let days_diff = 30;
        let today = NEXT_SERVICE.date() - TimeDelta::days(days_diff);

        let cmd = Status {
            curr_miles: miles,
            id: Some(ID.into()),
            today,
        };

        let log = build_log();
        let found = cmd.run(&log).unwrap();

        let mut buf = String::new();
        output_string_to_buf(&mut buf, miles, today, miles_diff, days_diff);

        assert_eq!(found, buf);
    }

    #[test]
    fn status_status_result() {
        let name = "name";
        let curr_miles = 1000;
        let next_service_miles = 1500;
        let date = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
        let next_service_date = NaiveDate::from_ymd_opt(2027, 7, 1).unwrap();

        let status_result = StatusResult {
            name,
            curr_miles,
            next_service_miles,
            date,
            next_service_date,
        };

        let miles_diff = next_service_miles as i64 - curr_miles as i64;
        assert_eq!(status_result.miles_diff(), miles_diff);

        let days_diff = (next_service_date - date).num_days();
        assert_eq!(status_result.days_diff(), days_diff);

        let mut buf = String::new();

        // String writes can't fail
        _ = writeln!(buf, "Name: {}", name);
        _ = writeln!(buf, "Next service (miles): {} Miles", next_service_miles);
        _ = writeln!(buf, "Current mileage: {} Miles", curr_miles);
        _ = writeln!(
            buf,
            "Miles until next service (Next Service Miles - Current Miles): {} Miles",
            miles_diff
        );
        _ = writeln!(buf, "Next service date: {}", next_service_date);
        _ = writeln!(buf, "Today: {}", date);
        _ = write!(
            buf,
            "Days until next service (Next Service Date - Today): {} Days",
            days_diff
        );

        assert_eq!(format!("{status_result}"), buf);
    }

    fn output_string_to_buf(
        buf: &mut String,
        miles: u32,
        today: NaiveDate,
        miles_diff: i32,
        days_diff: i64,
    ) {
        // Writes to a string can't fail
        _ = writeln!(buf, "Name: {}", NAME);
        _ = writeln!(buf, "Next service (miles): {} Miles", NEXT_SERVICE.miles());
        _ = writeln!(buf, "Current mileage: {} Miles", miles);
        _ = writeln!(
            buf,
            "Miles until next service (Next Service Miles - Current Miles): {} Miles",
            miles_diff
        );
        _ = writeln!(buf, "Next service date: {}", NEXT_SERVICE.date());
        _ = writeln!(buf, "Today: {}", today);
        _ = write!(
            buf,
            "Days until next service (Next Service Date - Today): {} Days",
            days_diff
        );
    }
}
