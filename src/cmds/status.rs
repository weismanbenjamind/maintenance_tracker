//! # Status
//!
//! Houses command for getting status of a single ID or multiple IDs

use std::fmt::{Display, Write};

use bon::Builder;
use chrono::{Local, NaiveDate};
use clap::Args;
use log::{debug, info};
use owo_colors::{OwoColorize, Style, style};

use crate::{
    containers::{MaintenanceLog, ServiceInterval},
    dates::days_to_months,
    errors::CmdsError,
};

const RED_THRESH: f64 = 1.00;
const YELLOW_THRESH: f64 = 0.75;
const RED: Style = style().red();
const YELLOW: Style = style().yellow();
const GREEN: Style = style().green();

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
    /// Build a new Status struct
    /// Only used for testing
    #[cfg(test)]
    pub(crate) fn new(curr_miles: u32, id: Option<String>, today: Option<NaiveDate>) -> Self {
        Self {
            curr_miles,
            id,
            today: today.unwrap_or(chrono::Local::now().date_naive()),
        }
    }

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
            let prev_service = m.prev_services().get_most_recent();

            let status_result = StatusResult::builder()
                .name(m.name())
                .curr_miles(self.curr_miles)
                .next_service_miles(next_service.miles())
                .date(self.today)
                .next_service_date(next_service.date())
                .service_interval(m.service_interval())
                .maybe_prev_service_miles(prev_service.map(|p| p.miles()))
                .maybe_prev_service_date(prev_service.map(|p| p.date()))
                .build();

            // Writing to a string can't fail
            _ = writeln!(buf, "{status_result}\n");
        });

        info!("Status operation complete");

        Ok(buf.trim_end().into())
    }
}

// TODO - How many of these fields do I need? Is using a struct here the best method?
/// View struct which houses the output of a status calculation.
/// Allows for easy formatting.
#[derive(Clone, Copy, Debug, Builder)]
struct StatusResult<'a> {
    name: &'a str,
    curr_miles: u32,
    next_service_miles: u32,
    date: NaiveDate,
    next_service_date: NaiveDate,
    service_interval: ServiceInterval,
    prev_service_miles: Option<u32>,
    prev_service_date: Option<NaiveDate>,
}

impl<'a> StatusResult<'a> {
    /// Get the miles differences between the next service miles
    /// and current miles on the vehicle.
    fn miles_diff(&self) -> i64 {
        self.next_service_miles as i64 - self.curr_miles as i64
    }

    /// Get the difference in days between the next service date
    /// and the date passed to the StatusResult struct.
    ///
    /// Assumes 365.25 days/year.
    ///
    /// Rounds to hundreths of a month.
    fn months_diff(&self) -> f64 {
        days_to_months(self.days_diff())
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
        writeln!(f, "Next service date: {}", self.next_service_date)?;
        writeln!(f, "Today: {}", self.date)?;

        let miles_ratio = calc_miles_ratio(
            self.curr_miles,
            self.next_service_miles,
            self.service_interval.miles(),
            self.prev_service_miles,
        );
        write_miles_output(f, miles_ratio, self.miles_diff())?;

        let time_ratio = calc_time_ratio(
            self.date,
            self.next_service_date,
            self.service_interval.days(),
            self.prev_service_date,
        );
        write_time_output(f, time_ratio, self.days_diff(), self.months_diff())
    }
}

fn calc_miles_ratio(
    curr_miles: u32,
    next_service_miles: u32,
    miles_interval: u32,
    prev_service_miles: Option<u32>,
) -> f64 {
    let diff = match prev_service_miles {
        Some(prev_service_miles) => curr_miles - prev_service_miles,
        None => next_service_miles - curr_miles,
    };

    diff as f64 / miles_interval as f64
}

fn write_miles_output(
    f: &mut std::fmt::Formatter,
    miles_ratio: f64,
    miles_diff: i64,
) -> std::fmt::Result {
    let style = get_style(miles_ratio);
    let mut writer = StyleWriter { f, style };

    writer.write("Distance to next service (Next service - Current): ")?;
    writer.write(miles_diff)?;
    writer.write(" Miles")?;
    writer.write(" (")?;
    writer.write(get_percentage(miles_ratio))?;
    writer.write("% of maintenance interval)")?;
    writer.newline()
}

fn calc_time_ratio(
    today: NaiveDate,
    next_service_date: NaiveDate,
    days_interval: i64,
    prev_service_date: Option<NaiveDate>,
) -> f64 {
    let diff = match prev_service_date {
        Some(prev_service_date) => today - prev_service_date,
        None => next_service_date - today,
    };

    diff.num_days() as f64 / days_interval as f64
}

fn write_time_output(
    f: &mut std::fmt::Formatter,
    time_ratio: f64,
    days_diff: i64,
    months_diff: f64,
) -> std::fmt::Result {
    let style = get_style(time_ratio);
    let mut writer = StyleWriter { f, style };

    writer.write("Days until next service (Next Service - Today): ")?;
    writer.write(days_diff)?;
    writer.write(" Days (")?;
    writer.write(months_diff)?;
    writer.write(" Months, ")?;
    writer.write(get_percentage(time_ratio))?;
    writer.write("% of maintenance inverval)")
}

fn get_style(ratio: f64) -> Style {
    if ratio >= RED_THRESH {
        RED
    } else if ratio >= YELLOW_THRESH {
        YELLOW
    } else {
        GREEN
    }
}

fn get_percentage(ratio: f64) -> f64 {
    (ratio * 10000.0).round() / 100.0
}

struct StyleWriter<'a, 'b> {
    f: &'a mut std::fmt::Formatter<'b>,
    style: Style,
}

impl<'a, 'b> StyleWriter<'a, 'b> {
    fn write<T: Display>(&mut self, t: T) -> std::fmt::Result {
        write!(self.f, "{}", t.style(self.style))
    }

    fn newline(&mut self) -> std::fmt::Result {
        writeln!(self.f)
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;

    use super::*;
    use crate::testing::{
        build_log, build_metadata,
        constants::{ID, NAME, NEXT_SERVICE},
    };

    #[test]
    fn status_new() {
        let curr_miles = 70000;
        let id = ID;
        let today = NaiveDate::from_ymd_opt(2026, 6, 15);

        let status = Status::new(curr_miles, Some(id.into()), today);
        assert_eq!(status.curr_miles, curr_miles);
        assert_eq!(&status.id.unwrap(), ID);
        assert_eq!(status.today, today.unwrap());
    }

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
        let months_diff = 0.99;
        let today = NEXT_SERVICE.date() - TimeDelta::days(days_diff);

        let cmd = Status {
            curr_miles: miles,
            id: None,
            today,
        };

        let found = cmd.run(&log).unwrap();

        let mut buf = String::new();
        output_string_to_buf(&mut buf, miles, today, miles_diff, days_diff, months_diff);
        _ = writeln!(buf);
        _ = writeln!(buf);
        output_string_to_buf(&mut buf, miles, today, miles_diff, days_diff, months_diff);

        assert_eq!(found, buf);
    }

    #[test]
    fn status_run_single_id() {
        let miles_diff: i32 = 3000;
        let miles = NEXT_SERVICE.miles() - miles_diff as u32;

        let days_diff = 30;
        let months_diff = 0.99;
        let today = NEXT_SERVICE.date() - TimeDelta::days(days_diff);

        let cmd = Status {
            curr_miles: miles,
            id: Some(ID.into()),
            today,
        };

        let log = build_log();
        let found = cmd.run(&log).unwrap();

        let mut buf = String::new();
        output_string_to_buf(&mut buf, miles, today, miles_diff, days_diff, months_diff);

        assert_eq!(found, buf);
    }

    // #[test]
    // fn status_status_result() {
    //     let name = "name";
    //     let curr_miles = 1000;
    //     let next_service_miles = 1500;
    //     let date = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
    //     let next_service_date = NaiveDate::from_ymd_opt(2027, 7, 1).unwrap();

    //     let status_result = StatusResult {
    //         name,
    //         curr_miles,
    //         next_service_miles,
    //         date,
    //         next_service_date,
    //     };

    //     let miles_diff = next_service_miles as i64 - curr_miles as i64;
    //     assert_eq!(status_result.miles_diff(), miles_diff);

    //     let days_diff = (next_service_date - date).num_days();
    //     assert_eq!(status_result.days_diff(), days_diff);

    //     // Manuallt calculated the months diff below
    //     let months_diff = 11.99;
    //     assert_eq!(months_diff, status_result.months_diff());

    //     let mut buf = String::new();

    //     // String writes can't fail
    //     _ = writeln!(buf, "Name: {}", name);
    //     _ = writeln!(buf, "Next service (miles): {} Miles", next_service_miles);
    //     _ = writeln!(buf, "Current mileage: {} Miles", curr_miles);
    //     _ = writeln!(
    //         buf,
    //         "Miles until next service (Next Service Miles - Current Miles): {} Miles",
    //         miles_diff
    //     );
    //     _ = writeln!(buf, "Next service date: {}", next_service_date);
    //     _ = writeln!(buf, "Today: {}", date);
    //     _ = writeln!(
    //         buf,
    //         "Days until next service (Next Service Date - Today): {} Days",
    //         days_diff
    //     );
    //     _ = write!(
    //         buf,
    //         "Months until next service (Next Service Date - Today): {months_diff} Months",
    //     );

    //     assert_eq!(format!("{status_result}"), buf);
    // }

    fn output_string_to_buf(
        buf: &mut String,
        miles: u32,
        today: NaiveDate,
        miles_diff: i32,
        days_diff: i64,
        months_diff: f64,
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
        _ = writeln!(
            buf,
            "Days until next service (Next Service Date - Today): {} Days",
            days_diff
        );
        _ = write!(
            buf,
            "Months until next service (Next Service Date - Today): {months_diff} Months",
        );
    }
}
