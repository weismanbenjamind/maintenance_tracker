use crate::cfg_resolve::resolve_cfg;
use crate::cli::MaintenanceTrackerArgs;
use crate::containers::MaintenanceLog;
use crate::errors::MaintenanceTrackerError;
use crate::verbosity::set_verbosity;
use log::info;

pub fn run(args: MaintenanceTrackerArgs) -> Result<(), MaintenanceTrackerError> {
    info!("Starting maintenance tracking run.");

    set_verbosity(args.verbose())?;
    let maintenance_log_path = resolve_cfg(args.maintenance_log(), args.maintenance_log_env())?;
    let log = MaintenanceLog::load(maintenance_log_path)?;
    println!("{:#?}", log);

    info!("Maintenance tracking run complete.");

    Ok(())
}
