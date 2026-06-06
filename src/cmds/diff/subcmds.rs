use crate::dates::months_to_days_floored;
use crate::{cmds::diff::Cmd as DiffCmd, errors::DiffError};
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

impl TryFrom<Interval> for Threshold {
    type Error = DiffError;

    fn try_from(value: Interval) -> Result<Self, Self::Error> {
        let (miles, curr_miles) = match (value.miles, value.curr_miles) {
            (Some(miles), Some(curr_miles)) => (Some(miles + curr_miles), Some(curr_miles)),
            (None, None) => (None, None),
            _ => return Err(DiffError::InvalidIntervalArgs),
        };

        let date = value.months.map(|months| {
            let days = months_to_days_floored(months);
            value.today + TimeDelta::days(days)
        });

        Ok(Threshold {
            miles: miles,
            date,
            curr_miles: curr_miles,
        })
    }
}

impl TryFrom<DiffCmd> for Threshold {
    type Error = DiffError;

    fn try_from(value: DiffCmd) -> Result<Self, Self::Error> {
        match value {
            DiffCmd::Threshold(threshold) => Ok(threshold),
            DiffCmd::Interval(interval) => interval.try_into(),
        }
    }
}
