//! # Diff Caclulator
//!
//! Used to calculate diffs in service miles/date thresholds vs. target miles/date respectivly.
//! Can also calculate diffs for service miles/date thresholds vs. target miles/date combinations.

use super::subcmds::ValidatedThreshold;

use chrono::NaiveDate;
use log::debug;

use crate::containers::ServiceMetdata;
use crate::dates::days_to_months;

/// Calculates service diffs.
///
/// Can calculate
/// - target miles vs. service miles
/// - target date vs. service date
/// - curr miles and target date vs. service miles and service date
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
    /// Calculate a diff between target miles, date, or miles/date combination
    /// and service miles, date, or miles/date combination.
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
                DiffResult::Time(Self::calc_time_diff(next_service_date, *date_threshold))
            }
            Self::MilesAndDate {
                curr_miles,
                date_threshold,
            } => DiffResult::MilesAndTime(
                Self::calc_miles_diff(next_service_miles, *curr_miles),
                Self::calc_time_diff(next_service_date, *date_threshold),
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

    /// Caclulate the miles diff between next service miles and current miles on the vehicle.
    fn calc_miles_diff(next_service_miles: u32, curr_miles: u32) -> MilesDiffResult {
        debug!(
            "Calculating miles diff with next service at {next_service_miles} miles and current miles at {curr_miles}"
        );
        MilesDiffResult {
            miles_diff: next_service_miles as i64 - curr_miles as i64,
            curr_miles,
        }
    }

    /// Calculate the time diff between next service date and a target date.
    fn calc_time_diff(next_service_date: NaiveDate, date_threshold: NaiveDate) -> TimeDiffResult {
        debug!(
            "Calculating time diff with next service date on {next_service_date} and date threshold {date_threshold}"
        );
        TimeDiffResult {
            days_diff: (next_service_date - date_threshold).num_days(),
            date_threshold,
        }
    }
}

/// Enum to house the output of a diff operation.
pub(super) struct DiffOutput<'a> {
    name: &'a str,
    next_service_miles: u32,
    next_service_date: NaiveDate,
    diff_result: DiffResult,
}

/// Enum to house the result of a diff operation for:
/// - miles
/// - time
/// - miles and time
#[derive(Clone, Copy, Debug, PartialEq)]
enum DiffResult {
    Miles(MilesDiffResult),
    Time(TimeDiffResult),
    MilesAndTime(MilesDiffResult, TimeDiffResult),
}

/// Struct to house the result of a miles diff.
#[derive(Clone, Copy, Debug, PartialEq)]
struct MilesDiffResult {
    miles_diff: i64,
    curr_miles: u32,
}

/// Struct to house the result of a time diff.
#[derive(Clone, Copy, Debug, PartialEq)]
struct TimeDiffResult {
    days_diff: i64,
    date_threshold: NaiveDate,
}

impl TimeDiffResult {
    /// Calculate the months diff from the days diff.
    ///
    /// Assumes 365.25 days/year.
    ///
    /// Rounds to nearest hundreths of a months.
    fn months_diff(&self) -> f64 {
        days_to_months(self.days_diff)
    }
}

impl<'a> std::fmt::Display for DiffOutput<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Name: {}", self.name)?;
        match &self.diff_result {
            DiffResult::Miles(miles_diff_result) => {
                write_miles_diff_result(f, *miles_diff_result, self.next_service_miles)
            }
            DiffResult::Time(days_diff_result) => {
                write_time_diff_result(f, *days_diff_result, self.next_service_date)
            }
            DiffResult::MilesAndTime(miles_diff_result, days_diff_result) => {
                write_miles_diff_result(f, *miles_diff_result, self.next_service_miles)?;
                writeln!(f)?;
                write_time_diff_result(f, *days_diff_result, self.next_service_date)
            }
        }
    }
}

/// Write a miles diff result to a Formatter.
fn write_miles_diff_result(
    f: &mut std::fmt::Formatter<'_>,
    miles_diff_result: MilesDiffResult,
    next_service_miles: u32,
) -> std::fmt::Result {
    writeln!(f, "Next service miles: {next_service_miles} Miles")?;
    writeln!(f, "Current miles: {} Miles", miles_diff_result.curr_miles)?;
    write!(
        f,
        "Miles until next service (Next Service - Current): {} Miles",
        miles_diff_result.miles_diff
    )
}

/// Write the time diff result to a Formatter.
fn write_time_diff_result(
    f: &mut std::fmt::Formatter<'_>,
    time_diff_result: TimeDiffResult,
    next_service_date: NaiveDate,
) -> std::fmt::Result {
    writeln!(f, "Next service date: {next_service_date}")?;
    writeln!(f, "Date threshold: {}", time_diff_result.date_threshold)?;
    writeln!(
        f,
        "Days until next service (Next Service Date - Date Threshold): {} Days",
        time_diff_result.days_diff
    )?;
    write!(
        f,
        "Months until next service (Next Service Date - Date Threshold): {} Months",
        time_diff_result.months_diff()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::build_metadata;
    use crate::testing::constants::{NAME, NEXT_SERVICE};
    use chrono::TimeDelta;

    #[test]
    fn diff_calculator_from_validated_threshold_miles_and_date() {
        let curr_miles = 500;
        let date_threshold = NaiveDate::from_ymd_opt(2026, 6, 14).unwrap();
        let thresh = ValidatedThreshold::MilesAndDate {
            miles_threshold: 1000,
            curr_miles,
            date_threshold,
        };
        let found = DiffCalculator::from(thresh);
        match found {
            DiffCalculator::MilesAndDate {
                curr_miles: found_curr_miles,
                date_threshold: found_date_threshold,
            } => {
                assert_eq!(curr_miles, found_curr_miles);
                assert_eq!(date_threshold, found_date_threshold);
            }
            _ => panic!("Expected DiffCalculator::MilesAndDate"),
        }
    }

    #[test]
    fn diff_calculator_from_validated_threshold_date() {
        let date_threshold = NaiveDate::from_ymd_opt(2026, 6, 14).unwrap();
        let thresh = ValidatedThreshold::Date { date_threshold };
        let found = DiffCalculator::from(thresh);
        match found {
            DiffCalculator::Date {
                date_threshold: found_date_threshold,
            } => assert_eq!(date_threshold, found_date_threshold),
            _ => panic!("Expected DiffCalculator::Date"),
        }
    }

    #[test]
    fn diff_calculator_from_validated_threshold_miles() {
        let curr_miles = 500;
        let thresh = ValidatedThreshold::Miles {
            miles_threshold: 1000,
            curr_miles: curr_miles,
        };
        let found = DiffCalculator::from(thresh);
        match found {
            DiffCalculator::Miles {
                curr_miles: found_curr_miles,
            } => assert_eq!(curr_miles, found_curr_miles),
            _ => panic!("Expected DiffCalculator::Miles"),
        }
    }

    #[test]
    fn diff_calculator_diff_miles() {
        let miles_diff = 1000;
        let curr_miles = NEXT_SERVICE.miles() - miles_diff;
        let calc = DiffCalculator::Miles { curr_miles };
        let metadata = build_metadata();

        let result = calc.diff(&metadata);

        assert_eq!(result.name, NAME);
        assert_eq!(result.next_service_miles, NEXT_SERVICE.miles());
        assert_eq!(result.next_service_date, NEXT_SERVICE.date());
        assert_eq!(
            result.diff_result,
            DiffResult::Miles(MilesDiffResult {
                miles_diff: miles_diff as i64,
                curr_miles
            })
        )
    }

    #[test]
    fn diff_calculator_diff_date() {
        let days_diff = 100;
        let date_threshold = NEXT_SERVICE.date() + TimeDelta::days(days_diff);
        let metadata = build_metadata();

        let calc = DiffCalculator::Date { date_threshold };
        let result = calc.diff(&metadata);

        assert_eq!(result.name, NAME);
        assert_eq!(result.next_service_miles, NEXT_SERVICE.miles());
        assert_eq!(result.next_service_date, NEXT_SERVICE.date());
        assert_eq!(
            result.diff_result,
            DiffResult::Time(TimeDiffResult {
                days_diff: -days_diff,
                date_threshold
            })
        );
    }

    #[test]
    fn diff_calculator_diff_miles_and_date() {
        let miles_diff = 1000;
        let curr_miles = NEXT_SERVICE.miles() - miles_diff;

        let days_diff = 100;
        let date_threshold = NEXT_SERVICE.date() + TimeDelta::days(days_diff);

        let calc = DiffCalculator::MilesAndDate {
            curr_miles,
            date_threshold,
        };
        let metadata = build_metadata();

        let result = calc.diff(&metadata);

        assert_eq!(result.name, NAME);
        assert_eq!(result.next_service_miles, NEXT_SERVICE.miles());
        assert_eq!(result.next_service_date, NEXT_SERVICE.date());

        let miles_diff_result = MilesDiffResult {
            miles_diff: miles_diff as i64,
            curr_miles,
        };
        let days_diff_result = TimeDiffResult {
            days_diff: -days_diff,
            date_threshold,
        };
        let diff_result = DiffResult::MilesAndTime(miles_diff_result, days_diff_result);

        assert_eq!(result.diff_result, diff_result);
    }

    #[test]
    fn diff_calculator_calc_miles_diff() {
        let miles_diff = 1000;
        let next_service_miles = 71000;
        let curr_miles = next_service_miles - miles_diff;
        let found = DiffCalculator::calc_miles_diff(next_service_miles, curr_miles);
        assert_eq!(found.curr_miles, curr_miles);
        assert_eq!(found.miles_diff, miles_diff as i64);
    }

    #[test]
    fn diff_calculator_calc_days_diff() {
        let days_diff = 30;
        let next_service_date = NaiveDate::from_ymd_opt(2026, 6, 3).unwrap();
        let date_threshold = next_service_date - TimeDelta::days(days_diff);
        let found = DiffCalculator::calc_time_diff(next_service_date, date_threshold);
        assert_eq!(found.date_threshold, date_threshold);
        assert_eq!(found.days_diff, days_diff);
    }

    #[test]
    fn diff_calculator_diff_output_display_miles() {
        let name = "name";
        let next_service_miles = 71000u32;
        let next_service_date = NaiveDate::from_ymd_opt(2026, 6, 3).unwrap();
        let miles_diff = 1000;
        let curr_miles = next_service_miles - miles_diff;
        let diff_result = DiffResult::Miles(MilesDiffResult {
            miles_diff: miles_diff as i64,
            curr_miles,
        });
        let diff_output = DiffOutput {
            name,
            next_service_miles,
            next_service_date,
            diff_result,
        };

        let found = format!("{diff_output}");
        let expected = format!(
            "Name: {name}\n\
            Next service miles: {next_service_miles} Miles\n\
            Current miles: {curr_miles} Miles\n\
            Miles until next service (Next Service - Current): {miles_diff} Miles",
        );
        assert_eq!(found, expected);
    }

    #[test]
    fn diff_calculator_diff_output_display_days() {
        let name = "name";
        let next_service_miles = 71000;
        let next_service_date = NaiveDate::from_ymd_opt(2026, 6, 3).unwrap();
        let days_diff = 30;
        let months_diff = 0.99;
        let date_threshold = next_service_date + TimeDelta::days(30);
        let diff_result = DiffResult::Time(TimeDiffResult {
            days_diff,
            date_threshold,
        });
        let diff_output = DiffOutput {
            name,
            next_service_miles,
            next_service_date,
            diff_result,
        };

        let found = format!("{diff_output}");
        let expected = format!(
            "Name: {name}\n\
            Next service date: {next_service_date}\n\
            Date threshold: {date_threshold}\n\
            Days until next service (Next Service Date - Date Threshold): {days_diff} Days\n\
            Months until next service (Next Service Date - Date Threshold): {months_diff} Months"
        );
        assert_eq!(found, expected);
    }

    #[test]
    fn diff_calculator_diff_output_display_miles_and_days() {
        let name = "name";

        let next_service_miles = 71000u32;
        let miles_diff = 1000;
        let curr_miles = next_service_miles - miles_diff;

        let next_service_date = NaiveDate::from_ymd_opt(2026, 6, 3).unwrap();
        let days_diff = 30;
        let months_diff = 0.99;
        let date_threshold = next_service_date + TimeDelta::days(30);

        let diff_result = DiffResult::MilesAndTime(
            MilesDiffResult {
                miles_diff: miles_diff as i64,
                curr_miles,
            },
            TimeDiffResult {
                days_diff,
                date_threshold,
            },
        );

        let diff_output = DiffOutput {
            name,
            next_service_miles,
            next_service_date,
            diff_result,
        };

        let found = format!("{diff_output}");
        let expected = format!(
            "Name: {name}\n\
            Next service miles: {next_service_miles} Miles\n\
            Current miles: {curr_miles} Miles\n\
            Miles until next service (Next Service - Current): {miles_diff} Miles\n\
            Next service date: {next_service_date}\n\
            Date threshold: {date_threshold}\n\
            Days until next service (Next Service Date - Date Threshold): {days_diff} Days\n\
            Months until next service (Next Service Date - Date Threshold): {months_diff} Months"
        );

        assert_eq!(found, expected);
    }

    struct TestWriteMilesDiffCase(MilesDiffResult, u32);

    impl std::fmt::Display for TestWriteMilesDiffCase {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write_miles_diff_result(f, self.0, self.1)
        }
    }

    #[test]
    fn diff_calculator_write_miles_diff() {
        let miles_diff = 1000;
        let curr_miles = 70000;

        let result = MilesDiffResult {
            miles_diff,
            curr_miles,
        };

        let next_service_miles = miles_diff as u32 + curr_miles;

        let found = format!("{}", TestWriteMilesDiffCase(result, next_service_miles));
        let expected = format!(
            "Next service miles: {next_service_miles} Miles\n\
            Current miles: {curr_miles} Miles\n\
            Miles until next service (Next Service - Current): {miles_diff} Miles",
        );

        assert_eq!(found, expected);
    }

    struct TestWriteDaysDiffCase(TimeDiffResult, NaiveDate);

    impl std::fmt::Display for TestWriteDaysDiffCase {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write_time_diff_result(f, self.0, self.1)
        }
    }

    #[test]
    fn diff_calculator_write_days_diff() {
        let date_threshold = NaiveDate::from_ymd_opt(2026, 6, 3).unwrap();
        let result = TimeDiffResult {
            days_diff: 30,
            date_threshold,
        };
        let next_service_date = date_threshold + chrono::TimeDelta::days(30);
        let months_diff = 0.99;

        let found = format!("{}", TestWriteDaysDiffCase(result, next_service_date));
        let expected = format!(
            "Next service date: {next_service_date}\n\
            Date threshold: {}\n\
            Days until next service (Next Service Date - Date Threshold): {} Days\n\
            Months until next service (Next Service Date - Date Threshold): {} Months",
            result.date_threshold, result.days_diff, months_diff,
        );

        assert_eq!(found, expected);
    }

    #[test]
    fn test_time_diff_result_months() {
        let time_diff_result = TimeDiffResult {
            days_diff: 45,
            date_threshold: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        };

        assert_eq!(time_diff_result.months_diff(), 1.48);
    }
}
