mod subcmds;

use chrono::NaiveDate;
use clap::{Args, Subcommand};
use log::{debug, info};
use std::fmt::Write;
use std::io::{self, Write as WriteStdOut};

use crate::containers::{MaintenanceLog, ServiceMetdata};
use crate::errors::{CmdsError, DiffError};

#[derive(Clone, Debug, Args)]
#[command(about = "Get info on services due by mileage and/or date intervals")]
pub struct Diff {
    #[command(subcommand)]
    cmd: Cmd,
}

impl Diff {
    pub fn run(self, log: &MaintenanceLog) -> Result<(), CmdsError> {
        info!("Running diff");

        let threshold = subcmds::Threshold::try_from(self.cmd)?;
        debug!("Using Threshold {:?}", threshold);

        let filter = MetadataFilter::try_from(threshold)?;
        debug!("Using MetadataFilter {:?}", filter);

        let calc = DiffCalculator::try_from(threshold)?;
        debug!("Using DiffCalculator {:?}", calc);

        let mut buf = String::new();

        // Writng to a string won't fail so unwrap below - TODO - is this statement true?
        log.metadata()
            .filter(|m| filter.apply(m))
            .map(|m| calc.diff(m))
            .for_each(|d| writeln!(buf, "{d}").unwrap());

        let stdout = io::stdout();
        let mut std_out_buf = stdout.lock();

        // TODO - Remove expects
        match buf.is_empty() {
            true => writeln!(std_out_buf, "No services due")
                .expect("Failed to write no services to stdout"),
            false => writeln!(std_out_buf, "\n{buf}").expect("Failed to write diff to stdout"),
        }

        info!("Finished running diff");
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Subcommand)]
enum Cmd {
    Threshold(subcmds::Threshold),
    Interval(subcmds::Interval),
}

#[derive(Debug, Clone, Copy)]
enum MetadataFilter {
    Miles(u32),
    Date(NaiveDate),
    MilesAndDate(u32, NaiveDate),
}

impl MetadataFilter {
    fn apply(&self, metadata: &ServiceMetdata) -> bool {
        let next_service = metadata.next_service();
        debug!("Filtering service {:?}", next_service);

        // TODO - maybe some quick helpers here to keep the code dry
        let result = match self {
            Self::Miles(miles) => {
                let next_service_miles = next_service.miles();
                debug!(
                    "Filtering where next service miles {next_service_miles} <= threshold miles {miles}"
                );
                next_service_miles <= *miles // Keep if the next service miles is below threshold
            }
            Self::Date(date) => {
                let next_service_date = next_service.date();
                debug!(
                    "Filtering where next service date {next_service_date} <= threshold date {date}"
                );
                next_service_date <= *date // Keep if next service date is below threshold
            }
            Self::MilesAndDate(miles, date) => {
                let next_service_miles = next_service.miles();
                let next_service_date = next_service.date();
                debug!(
                    "Filtering where next service miles {next_service_miles} <= threshold miles {miles}\
                    or next service date {next_service_date} <= threshold date {date}"
                );
                next_service.miles() <= *miles || next_service.date() <= *date // Keep if next service miles or next service date is below threshold
            }
        };
        debug!("Filter result {result}");
        result
    }
}

// TODO - from threshold might be a trait
impl TryFrom<subcmds::Threshold> for MetadataFilter {
    type Error = DiffError;

    fn try_from(value: subcmds::Threshold) -> Result<Self, Self::Error> {
        let filter = match (value.miles(), value.curr_miles(), value.date()) {
            (Some(miles), Some(_), None) => Self::Miles(miles),
            (None, None, Some(date)) => Self::Date(date),
            (Some(miles), Some(_), Some(date)) => Self::MilesAndDate(miles, date),
            _ => return Err(DiffError::InvalidThresholdArgs),
        };
        Ok(filter)
    }
}

#[derive(Debug, Clone, Copy)]
enum DiffCalculator {
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

// TODO - from threshold might be a trait
impl TryFrom<subcmds::Threshold> for DiffCalculator {
    type Error = DiffError;

    // TODO - vs try_from everwhere might want to parse the subcmd once into a validated state (enum potentially) then just build from that vs these try_froms everywhere
    fn try_from(value: subcmds::Threshold) -> Result<Self, Self::Error> {
        let calc = match (value.miles(), value.curr_miles(), value.date()) {
            (Some(_), Some(curr_miles), None) => Self::Miles { curr_miles },
            (None, None, Some(date_threshold)) => Self::Date { date_threshold },
            (Some(_), Some(curr_miles), Some(date_threshold)) => Self::MilesAndDate {
                curr_miles,
                date_threshold,
            },
            _ => return Err(DiffError::InvalidThresholdArgs),
        };
        Ok(calc)
    }
}

impl DiffCalculator {
    fn diff<'a>(&self, metadata: &'a ServiceMetdata) -> DiffOutput<'a> {
        debug!("Calculating diff for {:?}", metadata);
        // TODO - maybe get rid of these let statements
        let next_service = metadata.next_service();
        let next_service_miles = next_service.miles();
        let next_service_date = next_service.date();

        let diff_result = match self {
            Self::Miles { curr_miles } => {
                debug!("Calculating miles diff");
                DiffResult::Miles(Self::calc_miles_diff(next_service_miles, *curr_miles))
            }
            Self::Date { date_threshold } => {
                debug!("Calculating days diff");
                DiffResult::Days(Self::calc_days_diff(next_service_date, *date_threshold))
            }
            Self::MilesAndDate {
                curr_miles,
                date_threshold,
            } => {
                debug!("Calculating miles and days diff");
                let miles_diff = Self::calc_miles_diff(next_service_miles, *curr_miles);
                let days_diff = Self::calc_days_diff(next_service_date, *date_threshold);
                DiffResult::MilesAndDays(miles_diff, days_diff)
            }
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
        MilesDiffResult {
            miles_diff: next_service_miles - curr_miles,
            curr_miles,
        }
    }

    fn calc_days_diff(next_service_date: NaiveDate, date_threshold: NaiveDate) -> DaysDiffResult {
        DaysDiffResult {
            days_diff: (next_service_date - date_threshold).num_days(),
            date_threshold,
        }
    }
}

struct DiffOutput<'a> {
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
    miles_diff: u32,
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
                writeln!(f, "")?;
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
