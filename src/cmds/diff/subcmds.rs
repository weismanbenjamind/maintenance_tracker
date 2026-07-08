//! # Subcommands
//!
//! Houses subcommands for diff operations

use super::cmd::Cmd as DiffCmd;
use crate::dates::months_to_days_floored;
use crate::errors::DiffError;
use chrono::{Local, NaiveDate, TimeDelta};
use clap::Args;

/// Houses args for the Threshold subcommand
#[derive(Clone, Copy, Debug, Args)]
#[command(about = "Get all service requirements by a specific mileage and/or date")]
#[group(required = true, multiple = true)]
pub(super) struct Threshold {
    #[arg(
        short,
        long,
        requires = "curr_miles",
        help = "Miles threshold for maintenance. Requires that --curr-miles be passed"
    )]
    miles: Option<u32>,

    #[arg(
        short,
        long,
        requires = "miles",
        help = "Current miles on vehicle. Required if --miles is passed"
    )]
    curr_miles: Option<u32>,

    #[arg(short, long, help = "Date threshold for maintenance")]
    date: Option<NaiveDate>,
}

impl Threshold {
    /// Get a copy of the miles attribute
    pub(super) fn miles(&self) -> Option<u32> {
        self.miles
    }

    /// Get a copy of the date attribute
    pub(super) fn date(&self) -> Option<NaiveDate> {
        self.date
    }

    /// Get a copy of the current miles attribute
    pub(super) fn curr_miles(&self) -> Option<u32> {
        self.curr_miles
    }
}

/// Houses args for an Interval subcommand
#[derive(Clone, Copy, Debug, Args)]
#[command(
    about = "Get all service requirements from the current mileage and/or date due within some mileage and/or date interval"
)]
#[group(required = true, multiple = true)]
pub(super) struct Interval {
    #[arg(
        short,
        long,
        requires = "curr_miles",
        help = "Miles interval to check for maintenance. Requires --curr-miles be passed"
    )]
    miles: Option<u32>,

    #[arg(
        short,
        long,
        requires = "miles",
        help = "Current miles on vehicle. Required if --miles is passed"
    )]
    curr_miles: Option<u32>,

    #[arg(
        short = 'n',
        long,
        help = "Month interval to check for maintenance. By default checks from today. Use the --today arg to override the start date for this interval"
    )]
    months: Option<u32>,

    #[arg(short, long, default_value_t = Local::now().date_naive(), help = "Allows for overriding today's date if passing --months")]
    today: NaiveDate,
}

/// Enum which houses validated arguments for a Threshold diff
#[derive(Clone, Copy, Debug)]
pub(super) enum ValidatedThreshold {
    Miles {
        miles_threshold: u32,
        curr_miles: u32,
    },
    Date {
        date_threshold: NaiveDate,
    },
    MilesAndDate {
        miles_threshold: u32,
        curr_miles: u32,
        date_threshold: NaiveDate,
    },
}

impl TryFrom<Threshold> for ValidatedThreshold {
    type Error = DiffError;

    fn try_from(value: Threshold) -> Result<Self, Self::Error> {
        let result = match (value.miles(), value.curr_miles(), value.date()) {
            (Some(miles_threshold), Some(curr_miles), None) => Self::Miles {
                miles_threshold,
                curr_miles,
            },
            (None, None, Some(date_threshold)) => Self::Date { date_threshold },
            (Some(miles_threshold), Some(curr_miles), Some(date_threshold)) => Self::MilesAndDate {
                miles_threshold,
                curr_miles,
                date_threshold,
            },
            _ => return Err(DiffError::InvalidThresholdArgs),
        };
        Ok(result)
    }
}

impl TryFrom<Interval> for ValidatedThreshold {
    type Error = DiffError;

    fn try_from(value: Interval) -> Result<Self, Self::Error> {
        let result = match (value.miles, value.curr_miles, value.months) {
            (Some(miles), Some(curr_miles), None) => Self::Miles {
                miles_threshold: miles + curr_miles,
                curr_miles,
            },
            (None, None, Some(months)) => Self::Date {
                date_threshold: value.today + build_days_time_delta(months),
            },
            (Some(miles), Some(curr_miles), Some(months)) => Self::MilesAndDate {
                miles_threshold: miles + curr_miles,
                curr_miles,
                date_threshold: value.today + build_days_time_delta(months),
            },
            _ => return Err(DiffError::InvalidIntervalArgs),
        };
        Ok(result)
    }
}

/// Builds a time delta in days for a given amount of months.
/// Will floor the days computed from months if fractional
/// to avoid accidentally going over on maintenance.
fn build_days_time_delta(months: u32) -> TimeDelta {
    TimeDelta::days(months_to_days_floored(months))
}

impl TryFrom<DiffCmd> for ValidatedThreshold {
    type Error = DiffError;

    fn try_from(value: DiffCmd) -> Result<Self, Self::Error> {
        match value {
            DiffCmd::Threshold(threshold) => threshold.try_into(),
            DiffCmd::Interval(interval) => interval.try_into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subcmds_threshold_getters() {
        let thresh = Threshold {
            miles: Some(1000),
            curr_miles: Some(73000),
            date: None,
        };

        assert_eq!(thresh.miles(), Some(1000));
        assert_eq!(thresh.curr_miles(), Some(73000));
        assert!(thresh.date().is_none());
    }

    #[test]
    fn subcmds_try_from_interval_for_validated_thresh_miles() {
        let cmd = Interval {
            miles: Some(3000),
            curr_miles: Some(73000),
            months: None,
            today: NaiveDate::from_ymd_opt(2026, 6, 13).unwrap(),
        };

        let found: ValidatedThreshold = cmd.try_into().unwrap();

        match found {
            ValidatedThreshold::Miles {
                miles_threshold,
                curr_miles,
            } => {
                assert_eq!(
                    miles_threshold,
                    cmd.miles.unwrap() + cmd.curr_miles.unwrap()
                );
                assert_eq!(curr_miles, cmd.curr_miles.unwrap())
            }
            _ => panic!("Expected ValidatedThreshold::Miles"),
        }
    }

    #[test]
    fn subcmds_try_from_interval_for_validated_thresh_date() {
        let cmd = Interval {
            miles: None,
            curr_miles: None,
            months: Some(6),
            today: NaiveDate::from_ymd_opt(2026, 6, 13).unwrap(),
        };

        let found: ValidatedThreshold = cmd.try_into().unwrap();

        let timedelta = TimeDelta::days((cmd.months.unwrap() as f64 / 12.0 * 365.0).floor() as i64);
        let thresh = cmd.today + timedelta;

        match found {
            ValidatedThreshold::Date { date_threshold } => {
                assert_eq!(date_threshold, thresh);
            }
            _ => panic!("Expected ValidatedThreshold::Date"),
        }
    }

    #[test]
    fn subcmds_try_from_interval_for_validated_thresh_err() {
        let cmd = Interval {
            miles: None,
            curr_miles: None,
            months: None,
            today: NaiveDate::from_ymd_opt(2026, 6, 13).unwrap(),
        };

        let found = ValidatedThreshold::try_from(cmd).unwrap_err();

        match found {
            DiffError::InvalidIntervalArgs => (),
            _ => panic!("Expected DiffError::InvalidIntervalArgs"),
        }
    }

    #[test]
    fn subcmds_try_from_threshold_for_validated_thresh_err() {
        let cmd = Threshold {
            miles: None,
            curr_miles: None,
            date: None,
        };

        let found = ValidatedThreshold::try_from(cmd).unwrap_err();
        match found {
            DiffError::InvalidThresholdArgs => (),
            _ => panic!("Expected DiffError::InvalidThresholdArgs"),
        }
    }

    #[test]
    fn subcmds_try_from_threshold_for_validated_thresh_miles() {
        let cmd = Threshold {
            miles: Some(70000),
            curr_miles: Some(73000),
            date: None,
        };

        let found = ValidatedThreshold::try_from(cmd).unwrap();
        match found {
            ValidatedThreshold::Miles {
                miles_threshold: found_miles_thresh,
                curr_miles: found_curr_miles,
            } => {
                assert_eq!(found_miles_thresh, cmd.miles.unwrap());
                assert_eq!(found_curr_miles, cmd.curr_miles.unwrap());
            }
            _ => panic!("Expected ValidatedThreshold::Miles"),
        }
    }

    #[test]
    fn subcmds_try_from_threshold_for_validated_thresh_date() {
        let cmd = Threshold {
            miles: None,
            curr_miles: None,
            date: NaiveDate::from_ymd_opt(2026, 6, 13),
        };

        let found = ValidatedThreshold::try_from(cmd).unwrap();
        match found {
            ValidatedThreshold::Date {
                date_threshold: found_date_thresh,
            } => {
                assert_eq!(found_date_thresh, cmd.date.unwrap());
            }
            _ => panic!("Expected ValidatedThreshold::Date"),
        }
    }

    #[test]
    fn subcmds_try_from_diff_cmd_interval_validated_threshold() {
        let cmd = Interval {
            miles: Some(1000),
            curr_miles: Some(73000),
            months: Some(6),
            today: NaiveDate::from_ymd_opt(2026, 6, 13).unwrap(),
        };

        let found: ValidatedThreshold = cmd.try_into().unwrap();
        match found {
            ValidatedThreshold::MilesAndDate {
                miles_threshold: found_miles_thresh,
                curr_miles: found_curr_miles,
                date_threshold: found_date_thresh,
            } => {
                assert_eq!(
                    found_miles_thresh,
                    cmd.miles.unwrap() + cmd.curr_miles.unwrap()
                );
                assert_eq!(found_curr_miles, cmd.curr_miles.unwrap());
                assert_eq!(found_date_thresh, cmd.today + TimeDelta::days(182));
            }
            _ => panic!("Expected ValidatedThreshold::MilesAndDate"),
        }
    }

    #[test]
    fn subcmds_try_from_diff_cmd_threshold_validated_threshold() {
        let cmd = Threshold {
            miles: Some(70000),
            curr_miles: Some(73000),
            date: NaiveDate::from_ymd_opt(2026, 6, 13),
        };

        let found = ValidatedThreshold::try_from(cmd).unwrap();
        match found {
            ValidatedThreshold::MilesAndDate {
                miles_threshold: found_miles_thresh,
                curr_miles: found_curr_miles,
                date_threshold: found_date_thresh,
            } => {
                assert_eq!(found_miles_thresh, cmd.miles.unwrap());
                assert_eq!(found_curr_miles, cmd.curr_miles.unwrap());
                assert_eq!(found_date_thresh, cmd.date.unwrap());
            }
            _ => panic!("Expected ValidatedThreshold::MilesAndDate"),
        }
    }

    #[test]
    fn subcmds_build_time_delta_no_round() {
        assert_eq!(build_days_time_delta(36), TimeDelta::days(1095));
    }

    #[test]
    fn subcmds_build_time_delta_round() {
        assert_eq!(build_days_time_delta(35), TimeDelta::days(1064));
    }
}
