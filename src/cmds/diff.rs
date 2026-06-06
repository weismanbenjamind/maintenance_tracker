mod diff_calculator;
mod metadata_filter;
mod subcmds;

use diff_calculator::DiffCalculator;
use metadata_filter::MetadataFilter;
use subcmds::ValidatedThreshold;

use clap::{Args, Subcommand};
use log::{debug, info, warn};
use std::fmt::Write;
use std::io::{self, Write as WriteStdOut};

use crate::containers::MaintenanceLog;
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

        let threshold = ValidatedThreshold::try_from(self.cmd)?;
        debug!("Using ValidatedThreshold {:?}", threshold);

        let filter = MetadataFilter::from(threshold);
        debug!("Using MetadataFilter {:?}", filter);

        let calc = DiffCalculator::from(threshold);
        debug!("Using DiffCalculator {:?}", calc);

        let mut buf = String::new();

        // Writng to a string won't fail so unwrap below
        // writeln! below cannot fail when writing to a string
        //  Warn if a failure occurs but should never actually happen
        log.metadata()
            .filter(|m| filter.apply(m))
            .map(|m| calc.diff(m))
            .for_each(|d| writeln!(buf, "{d}").unwrap_or_else(|_| warn!("Failed to write Diff")));

        let stdout = io::stdout();
        let mut std_out_buf = stdout.lock();

        match buf.is_empty() {
            true => writeln!(std_out_buf, "No services due"),
            false => writeln!(std_out_buf, "\n{buf}"),
        }
        .map_err(DiffError::from)?;

        info!("Finished running diff");
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Subcommand)]
enum Cmd {
    Threshold(subcmds::Threshold),
    Interval(subcmds::Interval),
}
