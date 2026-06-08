use super::cmd::Cmd as DiffCmd;
use crate::dates::months_to_days_floored;
use crate::errors::DiffError;
use chrono::{Local, NaiveDate, TimeDelta};
use clap::Args;

// TODO - Might want to add a today override in here
#[derive(Clone, Copy, Debug, Args)]
#[command(about = "Get all service requirements by a specific mileage and/or date")]
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
    pub(super) fn miles(&self) -> Option<u32> {
        self.miles
    }

    pub(super) fn date(&self) -> Option<NaiveDate> {
        self.date
    }

    pub(super) fn curr_miles(&self) -> Option<u32> {
        self.curr_miles
    }
}

#[derive(Clone, Copy, Debug, Args)]
#[command(
    about = "Get all service requirements from the current mileage and/or date due within some mileage and/or date interval"
)]
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
