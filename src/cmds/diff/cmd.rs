//! # Command
//!
//! Houses top level command for diff operation

use super::diff_calculator::DiffCalculator;
use super::metadata_filter::MetadataFilter;
use super::subcmds;
use super::subcmds::ValidatedThreshold;

use clap::{Args, Subcommand};
use log::{debug, info};
use std::fmt::Write;

use crate::containers::MaintenanceLog;
use crate::errors::CmdsError;

/// Houses args for diff command
#[derive(Clone, Debug, Args)]
#[command(about = "Get info on services due by mileage and/or date intervals")]
pub(crate) struct Diff {
    #[arg(help = "Id to get diff for. If omitted diff will be executed for all maintenance items")]
    id: Option<String>,

    #[command(subcommand)]
    cmd: Cmd,
}

impl Diff {
    // TODO - Test this
    /// Create a new Diff Struct.
    /// Only used for testing.
    #[cfg(test)]
    pub(crate) fn new(id: Option<String>, cmd: Cmd) -> Self {
        Self { id, cmd }
    }

    /// Run the diff command
    pub(crate) fn run(self, log: &MaintenanceLog) -> Result<String, CmdsError> {
        info!("Running diff");

        let threshold = ValidatedThreshold::try_from(self.cmd)?;
        debug!("Using ValidatedThreshold {:?}", threshold);

        let filter = MetadataFilter::from(threshold);
        debug!("Using MetadataFilter {:?}", filter);

        let calc = DiffCalculator::from(threshold);
        debug!("Using DiffCalculator {:?}", calc);

        let mut buf = String::new();

        let target_metadata = match self.id {
            Some(id) => {
                vec![log.get(&id)?]
            }
            None => log.metadata_sorted(),
        };

        // Writng to a string won't fail so unwrap below
        // writeln! below cannot fail when writing to a string
        //  Warn if a failure occurs but should never actually happen
        target_metadata
            .iter()
            .filter(|m| filter.apply(m))
            .map(|m| calc.diff(m))
            // Writing to string cannot fail so unwrap below
            .for_each(|d| writeln!(buf, "{d}\n").unwrap());

        let msg = match buf.is_empty() {
            true => "No services due",
            false => buf.trim_end(),
        };

        info!("Finished running diff");
        Ok(msg.into())
    }
}

/// Enum for diff subcommands
#[derive(Clone, Copy, Debug, Subcommand)]
pub(crate) enum Cmd {
    Threshold(subcmds::Threshold),
    Interval(subcmds::Interval),
}

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;

    use super::*;
    use crate::testing::constants::{ID, NEXT_SERVICE};
    use crate::testing::{build_log, build_metadata};

    #[test]
    fn cmd_thresh_id() {
        let args = subcmds::Threshold::new(
            Some(NEXT_SERVICE.miles() + 1000),
            Some(NEXT_SERVICE.miles() - 1000),
            Some(NEXT_SERVICE.date() + TimeDelta::days(30)),
        )
        .unwrap();

        let cmd = Diff {
            id: Some(ID.into()),
            cmd: Cmd::Threshold(args),
        };

        let found = cmd.run(&build_log()).unwrap().to_lowercase();
        assert!(found.contains("miles until next service"));
        assert!(found.contains("days until next service"));
    }

    #[test]
    fn cmd_interval_id() {
        let args = subcmds::Interval::new(
            Some(1000),
            Some(NEXT_SERVICE.miles() - 10),
            Some(1),
            Some(NEXT_SERVICE.date() - TimeDelta::days(5)),
        )
        .unwrap();

        let cmd = Diff {
            id: Some(ID.into()),
            cmd: Cmd::Interval(args),
        };

        let found = cmd.run(&build_log()).unwrap().to_ascii_lowercase();
        assert!(found.contains("miles until next service"));
        assert!(found.contains("days until next service"));
    }

    #[test]
    fn cmd_missing_id() {
        let id = "other_id";
        assert_ne!(id, ID);

        let args = subcmds::Threshold::new(
            Some(NEXT_SERVICE.miles() + 1000),
            Some(NEXT_SERVICE.miles() - 1000),
            Some(NEXT_SERVICE.date() + TimeDelta::days(30)),
        )
        .unwrap();

        let cmd = Diff {
            id: Some(id.into()),
            cmd: Cmd::Threshold(args),
        };

        let found = cmd.run(&build_log()).unwrap_err();
        match found {
            CmdsError::IdNotFound(_) => (),
            _ => panic!("Expected CmdsError::IdNotFound"),
        }
    }

    #[test]
    fn cmd_no_services_due() {
        let args = subcmds::Threshold::new(
            Some(NEXT_SERVICE.miles() - 10),
            Some(70000),
            Some(NEXT_SERVICE.date() - TimeDelta::days(5)),
        )
        .unwrap();

        let cmd = Diff {
            id: Some(ID.into()),
            cmd: Cmd::Threshold(args),
        };

        let found = cmd.run(&build_log()).unwrap().to_ascii_lowercase();
        assert!(found.contains("no services due"));
    }

    #[test]
    fn cmd_no_id() {
        let id = "new_id";
        assert_ne!(id, ID);

        let metadata = build_metadata();
        let mut log = build_log();

        assert!(log.insert(id.into(), metadata).is_none());

        let args = subcmds::Threshold::new(
            Some(NEXT_SERVICE.miles() + 1000),
            Some(NEXT_SERVICE.miles() - 1000),
            Some(NEXT_SERVICE.date() + TimeDelta::days(30)),
        )
        .unwrap();

        let cmd = Diff {
            id: None,
            cmd: Cmd::Threshold(args),
        };

        let found = cmd.run(&log).unwrap().to_lowercase();
        assert_eq!(found.matches("miles until next service").count(), 2);
        assert_eq!(found.matches("days until next service").count(), 2);
    }
}
