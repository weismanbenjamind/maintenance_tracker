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

// Thresholds and styles for changing color of output text
const RED_THRESH: f64 = 1.00;
const YELLOW_THRESH: f64 = 0.75;
const RED: Style = style().red();
const YELLOW: Style = style().yellow();
const GREEN: Style = style().green();

// Max of u64 entirely fits max of i64
// So use u64 for type below (instead of i64)
// u64 is useful since that's what .unsigned_abs() returns in fits_f64 function
const MAX_I64_TO_F64: u64 = 1 << f64::MANTISSA_DIGITS;

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
                .maybe_prev_service(m.prev_service())
                .build();

            // Writing to a string can't fail
            _ = writeln!(buf, "{status_result}\n");
        });

        info!("Status operation complete");

        Ok(buf.trim_end().into())
    }
}

/// Ratio for maintenance intervals.
/// Used to ensure then denominator is not 0 when calculating the ratio.
/// Will warn if data loss occurs when ratio calc occus
/// (e.g. when `i64`s get converted to `f64`s in the ratio).
#[derive(Clone, Copy, Debug, PartialEq)]
struct MaintenanceIntervalRatio {
    numerator: i64,
    denominator: i64,
}

impl MaintenanceIntervalRatio {
    /// Safely calculates the maintenance interval ratio.
    /// Returns the ratio is the denominator is not zero,
    /// None otherwise
    fn safe_calc(&self) -> Option<f64> {
        (self.denominator != 0).then(|| {
            self.warn_if_data_loss();
            self.numerator as f64 / self.denominator as f64
        })
    }

    /// Emits a warning if data loss occurs when doing the `i64` -> `f64` conversions
    /// to calculate the maintenance interval ratio.
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

impl<'a> StatusResult<'a> {
    /// Get the miles differences between the next service miles
    /// and current miles on the vehicle.
    fn miles_diff(&self) -> i64 {
        i64::from(self.next_service.miles()) - i64::from(self.curr_miles)
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

    /// Calculates the miles ratio (% of miles used in a maintenance interval).
    fn miles_ratio(&self) -> MaintenanceIntervalRatio {
        let diff = match self.prev_service {
            Some(prev_service) => self.curr_miles - prev_service.miles(),
            None => self.curr_miles,
        };

        MaintenanceIntervalRatio {
            numerator: diff.into(),
            denominator: self.service_interval.miles().into(),
        }
    }

    /// Calculates the time ratio (% of time used in a maintenance interval).
    /// If no service has been performed yet, the time ratio is calculated by
    /// Back calculating t0 by subtracting the service interval from the expected first
    /// service date. t0 is then subtracted from the current date and this difference is
    /// divided by the maintenance interval.
    fn time_ratio(&self) -> MaintenanceIntervalRatio {
        let diff = match self.prev_service {
            Some(prev_service) => self.date - prev_service.date(),
            // If have not performed a service yet then t0
            // Is calculated by substracting the
            // service interval from the first service date
            None => {
                self.date
                    - (self.next_service.date() - TimeDelta::days(self.service_interval.days()))
            }
        };

        MaintenanceIntervalRatio {
            numerator: diff.num_days(),
            denominator: self.service_interval.days(),
        }
    }
}

impl<'a> std::fmt::Display for StatusResult<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Name: {}", self.name)?;

        let next_service = self.next_service;

        writeln!(f, "Next service: {} Miles", next_service.miles())?;
        writeln!(f, "Current mileage: {} Miles", self.curr_miles)?;
        writeln!(f, "Next service date: {}", next_service.date())?;
        writeln!(f, "Today: {}", self.date)?;
        write_miles_output(f, self.miles_ratio(), self.miles_diff())?;
        write_time_output(f, self.time_ratio(), self.days_diff(), self.months_diff())
    }
}

/// Struct to handle writing syled messags into a formatter.
struct StyleWriter<'a, 'b> {
    f: &'a mut std::fmt::Formatter<'b>,
    style: Option<Style>,
}

impl<'a, 'b> StyleWriter<'a, 'b> {
    /// Writes a styled message into a formatter if the .style attribute is not None.
    /// Otherwise, writes a non-styled message.
    /// No newline is written at the end of the message.
    fn write<T: Display>(&mut self, t: T) -> std::fmt::Result {
        match self.style {
            Some(s) => write!(self.f, "{}", t.style(s)),
            None => write!(self.f, "{}", t),
        }
    }

    /// Writes a newline into the self contained formatter.
    fn newline(&mut self) -> std::fmt::Result {
        writeln!(self.f)
    }
}

/// Checks if an i64 fits within an f64 completely.
/// Returns true if can convert without data loss,
/// false otherwise
fn fits_f64(i: i64) -> bool {
    // .unsigned_abs() will take the absolute value of i64
    // and convert to u64
    // Since u64::MAX > i64::MAX -> no data loss here
    i.unsigned_abs() < MAX_I64_TO_F64
}

/// Writes the miles output.
/// If the miles ratio can be safely calculated (e.g. a non-zero denominator)
/// The output will be written in red, yellow, or green depending on the % of the miles
/// interval used.
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

/// Writes the time output.
/// If the time ratio can be safely calculated (e.g. a non-zero denominator)
/// The output will be written in red, yellow, or green depending on the % of the time
/// interval used.
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
    writer.write("Time until next service (Next service - Today): ")?;
    writer.write(days_diff)?;
    writer.write(" Days/")?;
    writer.write(months_diff)?;
    writer.write(" Months")?;

    match time_ratio.safe_calc() {
        Some(ratio) => {
            writer.write(" (")?;
            writer.write(get_percentage(ratio))?;
            writer.write("% of maintenance interval)")?;
        }
        None => warn_for_zero_denom("time"),
    }

    Ok(())
}

/// Emits a warning if the denominator of a fraction is zero.
fn warn_for_zero_denom(interval_name: &str) {
    warn!("Found zero denominator when calculating percentage of {interval_name} used. Skipping.")
}

/// Gets the style used to write outputs which show % of interval used.
/// If the interval cannot be safely calculated returns None,
/// Otherwise returns the style (red, green, or yellow)
fn get_style(ratio: MaintenanceIntervalRatio) -> Option<Style> {
    ratio.safe_calc().map(|ratio| {
        if ratio >= RED_THRESH {
            RED
        } else if ratio >= YELLOW_THRESH {
            YELLOW
        } else {
            GREEN
        }
    })
}

/// Gets a percentage from a ratio. Percentage is rounded to 2 decimal places.
fn get_percentage(ratio: f64) -> f64 {
    (ratio * 10000.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;
    use owo_colors::{
        Color,
        colors::{Green as OWO_GREEN, Red as OWO_RED, Yellow as OWO_YELLOW},
    };

    use super::*;
    use crate::testing::{
        build_log, build_metadata,
        constants::{ID, NAME, NEXT_SERVICE, PREVIOUS_SERVICE, SERVICE_INTERVAL},
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

        let miles_diff: u32 = 3000;
        let miles = NEXT_SERVICE.miles() - miles_diff as u32;

        let miles_percent = (f64::from(miles - PREVIOUS_SERVICE.miles())
            / f64::from(SERVICE_INTERVAL.miles())
            * 10000.0)
            .round()
            / 100.00;

        let days_diff = 30;
        let months_diff = 0.99;
        let today = NEXT_SERVICE.date() - TimeDelta::days(days_diff);

        // Note `as f64` risks data loss
        let time_percent = ((today - PREVIOUS_SERVICE.date()).num_days() as f64
            / SERVICE_INTERVAL.days() as f64
            * 10000.0)
            .round()
            / 100.00;

        let cmd = Status {
            curr_miles: miles,
            id: None,
            today,
        };

        let found = cmd.run(&log).unwrap();
        let found = remove_ansi_colors(found);

        let mut buf = String::new();
        output_string_to_buf_percent(
            &mut buf,
            miles,
            today,
            miles_diff,
            days_diff,
            months_diff,
            miles_percent,
            time_percent,
        );

        _ = writeln!(buf);
        _ = writeln!(buf);

        output_string_to_buf_percent(
            &mut buf,
            miles,
            today,
            miles_diff,
            days_diff,
            months_diff,
            miles_percent,
            time_percent,
        );

        assert_eq!(found, buf);
    }

    #[test]
    fn status_run_single_id() {
        let miles_diff: u32 = 3000;
        let miles = NEXT_SERVICE.miles() - miles_diff;

        let miles_percent = (f64::from(miles - PREVIOUS_SERVICE.miles())
            / f64::from(SERVICE_INTERVAL.miles())
            * 10000.0)
            .round()
            / 100.00;

        let days_diff = 30;
        let months_diff = 0.99;
        let today = NEXT_SERVICE.date() - TimeDelta::days(days_diff);

        // Note `as f64` risks data loss
        let time_percent = ((today - PREVIOUS_SERVICE.date()).num_days() as f64
            / SERVICE_INTERVAL.days() as f64
            * 10000.0)
            .round()
            / 100.00;

        let cmd = Status {
            curr_miles: miles,
            id: Some(ID.into()),
            today,
        };

        let log = build_log();
        let found = cmd.run(&log).unwrap();
        let found = remove_ansi_colors(found);

        let mut buf = String::new();
        output_string_to_buf_percent(
            &mut buf,
            miles,
            today,
            miles_diff,
            days_diff,
            months_diff,
            miles_percent,
            time_percent,
        );

        assert_eq!(found, buf);
    }

    #[test]
    fn status_status_result() {
        let name = "name";
        let curr_miles = 1000;
        let service_interval_miles = 500;
        let next_service_miles = curr_miles + service_interval_miles;
        let date = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
        let next_service_date = NaiveDate::from_ymd_opt(2027, 7, 1).unwrap();
        let service_interval_days: u32 = 365;
        let service_interval_months = 12;

        let next_service = ServiceEvent::new(next_service_miles, next_service_date);
        let service_interval =
            ServiceInterval::new(service_interval_miles, service_interval_months);
        let prev_service = ServiceEvent::new(
            curr_miles - service_interval_miles,
            date - TimeDelta::days(i64::from(service_interval_days)),
        );

        let status_result = StatusResult::builder()
            .name(name)
            .curr_miles(curr_miles)
            .date(date)
            .next_service(next_service)
            .service_interval(service_interval)
            .prev_service(prev_service)
            .build();

        let miles_diff = i64::from(next_service_miles) - i64::from(curr_miles);
        assert_eq!(status_result.miles_diff(), miles_diff);

        let days_diff = (next_service_date - date).num_days();
        assert_eq!(status_result.days_diff(), days_diff);

        // Manually calculated the months diff below
        let months_diff = 11.99;
        assert_eq!(months_diff, status_result.months_diff());

        let expected_months_interval_ratio = MaintenanceIntervalRatio {
            numerator: i64::from(curr_miles - prev_service.miles()),
            denominator: i64::from(service_interval_miles),
        };
        assert_eq!(status_result.miles_ratio(), expected_months_interval_ratio);

        let expected_time_interval_ratio = MaintenanceIntervalRatio {
            numerator: (date - prev_service.date()).num_days(),
            denominator: i64::from(service_interval_days),
        };
        assert_eq!(status_result.time_ratio(), expected_time_interval_ratio);

        let miles_percent = (f64::from(curr_miles - prev_service.miles())
            / f64::from(service_interval.miles())
            * 10000.0)
            .round()
            / 100.00;

        // Note - risking data loss with as f64 notation below
        let time_percent = ((date - prev_service.date()).num_days() as f64
            / service_interval.days() as f64
            * 10000.0)
            .round()
            / 100.00;

        let mut buf = String::new();
        // Writes to a string can't fail
        _ = writeln!(buf, "Name: {}", name);
        _ = writeln!(buf, "Next service: {} Miles", next_service.miles());
        _ = writeln!(buf, "Current mileage: {} Miles", curr_miles);
        _ = writeln!(buf, "Next service date: {}", next_service.date());
        _ = writeln!(buf, "Today: {}", date);
        _ = writeln!(
            buf,
            "Distance to next service (Next service - Current): {} Miles ({}% of maintenance interval)",
            miles_diff, miles_percent
        );
        _ = write!(
            buf,
            "Time until next service (Next service - Today): {} Days/{} Months ({}% of maintenance interval)",
            days_diff, months_diff, time_percent
        );

        let found = format!("{status_result}");
        let found = remove_ansi_colors(found);

        assert_eq!(found, buf);
    }

    #[test]
    fn test_status_green() {
        let green_frac = 0.05;
        let miles =
            green_frac * f64::from(SERVICE_INTERVAL.miles()) + f64::from(PREVIOUS_SERVICE.miles());

        let days_diff = green_frac * SERVICE_INTERVAL.days() as f64; // Note - risking data loss
        let today = PREVIOUS_SERVICE.date() + TimeDelta::days(days_diff as i64); // Note - risking data loss

        let cmd = Status {
            curr_miles: miles as u32, // Note - risking data loss with as notation
            id: Some(ID.into()),
            today,
        };

        let log = build_log();
        let found = cmd.run(&log).unwrap();

        assert_eq!(found.matches("% of maintenance interval").count(), 2);
        assert!(found.contains(OWO_GREEN::ANSI_FG));
        assert!(!found.contains(OWO_YELLOW::ANSI_FG));
        assert!(!found.contains(OWO_RED::ANSI_FG));
    }

    #[test]
    fn test_status_yellow() {
        let yellow_frac = YELLOW_THRESH + 0.05;
        let miles =
            yellow_frac * f64::from(SERVICE_INTERVAL.miles()) + f64::from(PREVIOUS_SERVICE.miles());

        let days_diff = yellow_frac * SERVICE_INTERVAL.days() as f64; // Note - risking data loss
        let today = PREVIOUS_SERVICE.date() + TimeDelta::days(days_diff as i64); // Note - risking data loss

        let cmd = Status {
            curr_miles: miles as u32, // Note - risking data loss with as notation
            id: Some(ID.into()),
            today,
        };

        let log = build_log();
        let found = cmd.run(&log).unwrap();

        assert_eq!(found.matches("% of maintenance interval").count(), 2);
        assert!(found.contains(OWO_YELLOW::ANSI_FG));
        assert!(!found.contains(OWO_GREEN::ANSI_FG));
        assert!(!found.contains(OWO_RED::ANSI_FG));
    }

    #[test]
    fn test_status_red() {
        let miles = PREVIOUS_SERVICE.miles() + SERVICE_INTERVAL.miles() + 1;
        let today = PREVIOUS_SERVICE.date() + TimeDelta::days(SERVICE_INTERVAL.days() + 1);

        let cmd = Status {
            curr_miles: miles,
            id: Some(ID.into()),
            today,
        };

        let log = build_log();
        let found = cmd.run(&log).unwrap();

        assert_eq!(found.matches("% of maintenance interval").count(), 2);
        assert!(found.contains(OWO_RED::ANSI_FG));
        assert!(!found.contains(OWO_YELLOW::ANSI_FG));
        assert!(!found.contains(OWO_GREEN::ANSI_FG));
    }

    #[test]
    fn test_zero_denom_skips_color_and_percent_interval() {
        let mut log = build_log();
        let service_interval = log.get_mut(ID).unwrap().service_interval_mut();
        service_interval.set_miles(0);
        service_interval.set_months(0);

        let cmd = Status {
            curr_miles: NEXT_SERVICE.miles() - 100,
            id: Some(ID.into()),
            today: NEXT_SERVICE.date() - TimeDelta::days(30),
        };

        let found = cmd.run(&log).unwrap();

        assert_eq!(found.matches("% of maintenance interval").count(), 0);
        assert!(!found.contains(OWO_RED::ANSI_FG));
        assert!(!found.contains(OWO_YELLOW::ANSI_FG));
        assert!(!found.contains(OWO_GREEN::ANSI_FG));
    }

    #[test]
    fn test_no_prev_services() {
        let mut log = build_log();
        log.get_mut(ID).unwrap().prev_services_mut().clear();

        let curr_miles = NEXT_SERVICE.miles() - 100;
        let expected_miles_percent =
            (f64::from(curr_miles) / f64::from(SERVICE_INTERVAL.miles()) * 10000.0).round() / 100.0;

        let today = NEXT_SERVICE.date() - TimeDelta::days(30);
        let time_diff =
            (today - (NEXT_SERVICE.date() - TimeDelta::days(SERVICE_INTERVAL.days()))).num_days();
        let expected_time_precent =
            (time_diff as f64 / SERVICE_INTERVAL.days() as f64 * 10000.0).round() / 100.0;

        let cmd = Status {
            curr_miles: NEXT_SERVICE.miles() - 100,
            id: Some(ID.into()),
            today: NEXT_SERVICE.date() - TimeDelta::days(30),
        };

        let found = cmd.run(&log).unwrap();
        let found = remove_ansi_colors(found);

        assert!(found.contains(&format!(
            "{expected_miles_percent}% of maintenance interval"
        )));
        assert!(found.contains(&format!("{expected_time_precent}% of maintenance interval")));
    }

    fn output_string_to_buf_percent(
        buf: &mut String,
        miles: u32,
        today: NaiveDate,
        miles_diff: u32,
        days_diff: i64,
        months_diff: f64,
        miles_percent: f64,
        time_percent: f64,
    ) {
        // Writes to a string can't fail
        _ = writeln!(buf, "Name: {}", NAME);
        _ = writeln!(buf, "Next service: {} Miles", NEXT_SERVICE.miles());
        _ = writeln!(buf, "Current mileage: {} Miles", miles);
        _ = writeln!(buf, "Next service date: {}", NEXT_SERVICE.date());
        _ = writeln!(buf, "Today: {}", today);
        _ = writeln!(
            buf,
            "Distance to next service (Next service - Current): {} Miles ({}% of maintenance interval)",
            miles_diff, miles_percent
        );
        _ = write!(
            buf,
            "Time until next service (Next service - Today): {} Days/{} Months ({}% of maintenance interval)",
            days_diff, months_diff, time_percent
        );
    }

    fn remove_ansi_colors(s: String) -> String {
        let stripped = strip_ansi_escapes::strip(s);
        String::from_utf8(stripped).unwrap()
    }
}
