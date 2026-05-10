use std::path::{Path, PathBuf};

use crate::cli::MaintenanceTrackerArgs;
use crate::containers::MaintenanceLog;
use crate::errors::CfgResolveError;
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

fn resolve_cfg(
    maintenance_log: &Path,
    maintenance_log_env: &str,
) -> Result<PathBuf, CfgResolveError> {
    let result = match maintenance_log.exists() {
        true => Ok(maintenance_log.into()),
        false => match std::env::var(maintenance_log_env) {
            Ok(path) => Ok(PathBuf::from(path)),
            Err(_) => Err(CfgResolveError::ResolveFailure(
                maintenance_log.into(),
                maintenance_log_env.to_string(),
            )),
        },
    };

    if let Ok(path) = &result {
        info!("Resolve maintenance log path to {}.", path.display())
    }

    result
}
