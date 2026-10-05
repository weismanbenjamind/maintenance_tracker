//! # Status
//!
//! Houses command for getting status of a single ID or multiple IDs

use std::fmt::{Display, Write};

use bon::Builder;
use chrono::{Local, NaiveDate, TimeDelta};
use clap::Args;
use log::{debug, info, warn};
use owo_colors::{OwoColorize, Style, style};

use crate::{
    containers::{MaintenanceLog, ServiceEvent, ServiceInterval},
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
            let status_result = StatusResult::builder()
                .name(m.name())
                .curr_miles(self.curr_miles)
                .date(self.today)
                .next_service(m.next_service())
                .service_interval(m.service_interval())
                .maybe_prev_service(m.prev_services().get_most_recent())
                .build();

            // Writing to a string can't fail
            _ = writeln!(buf, "{status_result}\n");
        });

        info!("Status operation complete");

        Ok(buf.trim_end().into())
    }
}

// Max of u64 entirely fits max of i64
// So use u64 for type below (instead of i64)
// u64 is useful since that's what .unsigned_abs() returns in fits_f64 function
const MAX_I64_TO_F64: u64 = 1 << f64::MANTISSA_DIGITS;

fn fits_f64(i: i64) -> bool {
    // .unsigned_abs() will take the absolute value of i64
    // and convert to u64
    // Since u64::MAX > i64::MAX -> no data loss here
    i.unsigned_abs() < MAX_I64_TO_F64
}

#[derive(Clone, Copy, Debug, Builder)]
struct MaintenanceIntervalRatio {
    numerator: i64,
    denominator: i64,
}

impl MaintenanceIntervalRatio {
    fn safe_calc(&self) -> Option<f64> {
        (self.denominator != 0).then(|| {
            self.warn_if_data_loss();
            self.numerator as f64 / self.denominator as f64
        })
    }

    fn warn_if_data_loss(&self) {
        if !fits_f64(self.numerator) || !fits_f64(self.denominator) {
            warn!(
                "Detected data loss when calculating ratio for percentage of maintenance interval used"
            )
        }
    }
}

/// View struct which houses the output of a status calculation.
/// Allows for easy formatting.
#[derive(Clone, Copy, Debug, Builder)]
struct StatusResult<'a> {
    name: &'a str,
    curr_miles: u32,
    date: NaiveDate,
    next_service: ServiceEvent,
    service_interval: ServiceInterval,
    prev_service: Option<ServiceEvent>,
}

// TODO - Overflow and divide by zero guards here. Use ::From vs. `as` to guard against
// overflow
impl<'a> StatusResult<'a> {
    /// Get the miles differences between the next service miles
    /// and current miles on the vehicle.
    fn miles_diff(&self) -> i64 {
        self.next_service.miles() as i64 - self.curr_miles as i64
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
        (self.next_service.date() - self.date).num_days()
    }

    fn miles_ratio(&self) -> MaintenanceIntervalRatio {
        let diff = match self.prev_service.map(|p| p.miles()) {
            Some(prev_service_miles) => self.curr_miles - prev_service_miles,
            None => self.curr_miles,
        };

        MaintenanceIntervalRatio::builder()
            .numerator(diff.into())
            .denominator(self.service_interval.miles().into())
            .build()
    }

    fn time_ratio(&self) -> MaintenanceIntervalRatio {
        let diff = match self.prev_service.map(|p| p.date()) {
            Some(prev_service_date) => self.date - prev_service_date,
            // If have not performed a service yet then t0
            // Is calculated by substracting the
            // service interval from the first service date
            None => {
                self.date
                    - (self.next_service.date() - TimeDelta::days(self.service_interval.days()))
            }
        };

        MaintenanceIntervalRatio::builder()
            .numerator(diff.num_days())
            .denominator(self.service_interval.days())
            .build()
    }
}

impl<'a> std::fmt::Display for StatusResult<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Name: {}", self.name)?;

        let next_service = self.next_service;

        writeln!(f, "Next service (miles): {} Miles", next_service.miles())?;
        writeln!(f, "Current mileage: {} Miles", self.curr_miles)?;
        writeln!(f, "Next service date: {}", next_service.date())?;
        writeln!(f, "Today: {}", self.date)?;
        write_miles_output(f, self.miles_ratio(), self.miles_diff())?;
        write_time_output(f, self.time_ratio(), self.days_diff(), self.months_diff())
    }
}

fn write_miles_output(
    f: &mut std::fmt::Formatter,
    miles_ratio: MaintenanceIntervalRatio,
    miles_diff: i64,
) -> std::fmt::Result {
    // If miles ratio has a zero denominator style will be None
    // And style writer will write without any color
    let style = get_style(miles_ratio);
    let mut writer = StyleWriter { f, style };

    writer.write("Distance to next service (Next service - Current): ")?;
    writer.write(miles_diff)?;
    writer.write(" Miles")?;

    // If have a zero denominator warn and do not write % of miles interval used
    // Style writer will be configured to print without color if have a zero
    // denominator so text already printed above will be colored correctly
    match miles_ratio.safe_calc() {
        Some(ratio) => {
            writer.write(" (")?;
            writer.write(get_percentage(ratio))?;
            writer.write("% of maintenance interval)")?;
            writer.newline()?;
        }
        None => warn_for_zero_denom("miles"),
    }

    Ok(())
}

fn write_time_output(
    f: &mut std::fmt::Formatter,
    time_ratio: MaintenanceIntervalRatio,
    days_diff: i64,
    months_diff: f64,
) -> std::fmt::Result {
    // If time ratio has a zero denominator style will be None
    // And style writer will write without any color
    let style = get_style(time_ratio);
    let mut writer = StyleWriter { f, style };

    // If have a zero denominator warn and do not write % of time interval used
    // Style writer will be configured to print without color if have a zero
    // denominator so text already printed above will be colored correctly
    writer.write("Time until next service (Next Service - Today): ")?;
    writer.write(days_diff)?;
    writer.write(" Days/")?;
    writer.write(months_diff)?;
    writer.write(" Months")?;

    match time_ratio.safe_calc() {
        Some(ratio) => {
            writer.write(" (")?;
            writer.write(get_percentage(ratio))?;
            writer.write("% of maintenance inverval)")?;
        }
        None => warn_for_zero_denom("time"),
    }

    Ok(())
}

fn warn_for_zero_denom(interval_name: &str) {
    warn!("Found zero denominator when calculating percentage of {interval_name} used. Skipping.")
}

fn get_style(ratio: MaintenanceIntervalRatio) -> Option<Style> {
    ratio.safe_calc().map(_get_style)
}

fn _get_style(ratio: f64) -> Style {
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
    style: Option<Style>,
}

impl<'a, 'b> StyleWriter<'a, 'b> {
    fn write<T: Display>(&mut self, t: T) -> std::fmt::Result {
        match self.style {
            Some(s) => write!(self.f, "{}", t.style(s)),
            None => write!(self.f, "{}", t),
        }
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
