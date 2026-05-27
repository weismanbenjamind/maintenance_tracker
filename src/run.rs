use crate::cfg_resolve::resolve_cfg;
use crate::cli::Cmd;
use crate::cli::MaintenanceTrackerArgs;
use crate::containers::MaintenanceLog;
use crate::errors::MaintenanceTrackerError;
use crate::verbosity::set_verbosity;
use log::info;

pub fn run(args: MaintenanceTrackerArgs) -> Result<(), MaintenanceTrackerError> {
    info!("Starting maintenance tracking run.");

    set_verbosity(args.verbose())?;
    let maintenance_log_path = resolve_cfg(args.maintenance_log(), args.maintenance_log_env())?;
    let mut log = MaintenanceLog::load(maintenance_log_path)?;
    match args.into_cmd() {
        Cmd::List(list_cmd) => list_cmd.run(&log)?,
        Cmd::Next(next_cmd) => next_cmd.run(&log)?,
        Cmd::Init(init_cmd) => {
            init_cmd.run(&mut log)?;
            println!("{:#?}", log)
        }
        _ => println!("{:#?}", log),
    }

    info!("Maintenance tracking run complete.");

    Ok(())
}
