use super::subcmds::ValidatedThreshold;

use chrono::NaiveDate;
use log::debug;

use crate::containers::ServiceMetdata;

#[derive(Debug, Clone, Copy)]
pub(super) enum DiffCalculator {
    Miles {
        curr_miles: u32,
    },
    Date {
        date_threshold: NaiveDate,
    },
    MilesAndDate {
        curr_miles: u32,
        date_threshold: NaiveDate,
    },
}

impl From<ValidatedThreshold> for DiffCalculator {
    fn from(value: ValidatedThreshold) -> Self {
        match value {
            ValidatedThreshold::Miles { curr_miles, .. } => Self::Miles { curr_miles },
            ValidatedThreshold::Date { date_threshold } => Self::Date { date_threshold },
            ValidatedThreshold::MilesAndDate {
                curr_miles,
                date_threshold,
                ..
            } => Self::MilesAndDate {
                curr_miles,
                date_threshold,
            },
        }
    }
}

impl DiffCalculator {
    pub(super) fn diff<'a>(&self, metadata: &'a ServiceMetdata) -> DiffOutput<'a> {
        debug!("Calculating diff for {:?}", metadata);
        let next_service = metadata.next_service();
        let next_service_miles = next_service.miles();
        let next_service_date = next_service.date();

        let diff_result = match self {
            Self::Miles { curr_miles } => {
                DiffResult::Miles(Self::calc_miles_diff(next_service_miles, *curr_miles))
            }
            Self::Date { date_threshold } => {
                DiffResult::Days(Self::calc_days_diff(next_service_date, *date_threshold))
            }
            Self::MilesAndDate {
                curr_miles,
                date_threshold,
            } => DiffResult::MilesAndDays(
                Self::calc_miles_diff(next_service_miles, *curr_miles),
                Self::calc_days_diff(next_service_date, *date_threshold),
            ),
        };

        debug!("Diff result {:?}", diff_result);

        DiffOutput {
            name: metadata.name(),
            next_service_miles,
            next_service_date,
            diff_result,
        }
    }

    fn calc_miles_diff(next_service_miles: u32, curr_miles: u32) -> MilesDiffResult {
        debug!(
            "Calculating miles diff with next service at {next_service_miles} miles and current miles at {curr_miles}"
        );
        MilesDiffResult {
            miles_diff: next_service_miles as i64 - curr_miles as i64,
            curr_miles,
        }
    }

    fn calc_days_diff(next_service_date: NaiveDate, date_threshold: NaiveDate) -> DaysDiffResult {
        debug!(
            "Calculating days diff with next service date on {next_service_date} and date threshold {date_threshold}"
        );
        DaysDiffResult {
            days_diff: (next_service_date - date_threshold).num_days(),
            date_threshold,
        }
    }
}

pub(super) struct DiffOutput<'a> {
    name: &'a str,
    next_service_miles: u32,
    next_service_date: NaiveDate,
    diff_result: DiffResult,
}

#[derive(Clone, Copy, Debug)]
enum DiffResult {
    Miles(MilesDiffResult),
    Days(DaysDiffResult),
    MilesAndDays(MilesDiffResult, DaysDiffResult),
}

#[derive(Clone, Copy, Debug)]
struct MilesDiffResult {
    miles_diff: i64,
    curr_miles: u32,
}

#[derive(Clone, Copy, Debug)]
struct DaysDiffResult {
    days_diff: i64,
    date_threshold: NaiveDate,
}

impl<'a> std::fmt::Display for DiffOutput<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Name: {}", self.name)?;
        writeln!(f, "Next Service Miles: {}", self.next_service_miles)?;
        writeln!(f, "Next Service Date: {}", self.next_service_date)?;
        match &self.diff_result {
            DiffResult::Miles(miles_diff_result) => write_miles_diff_result(f, *miles_diff_result),
            DiffResult::Days(days_diff_result) => write_days_diff_result(f, *days_diff_result),
            DiffResult::MilesAndDays(miles_diff_result, days_diff_result) => {
                write_miles_diff_result(f, *miles_diff_result)?;
                writeln!(f)?;
                write_days_diff_result(f, *days_diff_result)
            }
        }
    }
}

fn write_miles_diff_result(
    f: &mut std::fmt::Formatter<'_>,
    miles_diff_result: MilesDiffResult,
) -> std::fmt::Result {
    writeln!(f, "Current Miles: {}", miles_diff_result.curr_miles)?;
    write!(
        f,
        "Miles until next service (Next Service - Current): {}",
        miles_diff_result.miles_diff
    )
}

fn write_days_diff_result(
    f: &mut std::fmt::Formatter<'_>,
    days_diff_result: DaysDiffResult,
) -> std::fmt::Result {
    writeln!(f, "Date Threshold: {}", days_diff_result.date_threshold)?;
    write!(
        f,
        "Days until next service (Next Service Date - Target Date): {}",
        days_diff_result.days_diff
    )
}
