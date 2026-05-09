use crate::cli::MaintenanceTrackerArgs;
use crate::containers::MaintenanceLog;

// ADD LOGGING!!!!!!!!!!!

pub fn run(args: MaintenanceTrackerArgs) -> Result<(), String> {
    let toml = std::fs::read_to_string(args.maintenance_log())
        .map_err(|e| format!("Failed to read maintenance log with error: {e}"))?;

    let log: MaintenanceLog = toml::from_str(&toml)
        .map_err(|e| format!("Failed to parse toml representing maintenance log. Error {e}"))?;

    println!("{:#?}", log);

    Ok(())
}
