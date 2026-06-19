use super::diff_calculator::DiffCalculator;
use super::metadata_filter::MetadataFilter;
use super::subcmds;
use super::subcmds::ValidatedThreshold;

use clap::{Args, Subcommand};
use log::{debug, info};
use std::fmt::Write;
use std::io::{self, Write as WriteStdOut};

use crate::containers::MaintenanceLog;
use crate::errors::CmdsError;

#[derive(Clone, Debug, Args)]
#[command(about = "Get info on services due by mileage and/or date intervals")]
pub(crate) struct Diff {
    #[arg(help = "Id to get diff for. If omitted diff will be executed for all maintenance items")]
    id: Option<String>,

    #[command(subcommand)]
    cmd: Cmd,
}

impl Diff {
    pub(crate) fn run(self, log: &MaintenanceLog) -> Result<(), CmdsError> {
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

        let stdout = io::stdout();
        let mut std_out_buf = stdout.lock();

        match buf.is_empty() {
            true => writeln!(std_out_buf, "No services due"),
            false => writeln!(std_out_buf, "{}", buf.trim_end()),
        }?;

        info!("Finished running diff");
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Subcommand)]
pub(super) enum Cmd {
    Threshold(subcmds::Threshold),
    Interval(subcmds::Interval),
}
