use crate::containers::ServiceMetdata;
use crate::errors::MaintenanceLogError;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::hash_map::Keys;
use std::path::Path;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MaintenanceLog {
    services: HashMap<String, ServiceMetdata>,
}

impl MaintenanceLog {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<MaintenanceLog, MaintenanceLogError> {
        let path = path.as_ref();
        info!("Reading maintenance log path at {}.", path.display());
        let log =
            std::fs::read_to_string(path).map_err(|source| MaintenanceLogError::FailedLoad {
                path: path.into(),
                source,
            })?;
        Ok(toml::from_str::<MaintenanceLog>(&log)?)
    }

    pub fn write<P: AsRef<Path>>(&self, path: P) -> Result<(), MaintenanceLogError> {
        let path = path.as_ref();
        info!("Writing maintenance log to {}.", path.display());
        let log_string = toml::to_string_pretty(&self)?;
        build_subdirs(path)?;
        std::fs::write(path, log_string)
            .map_err(|e| MaintenanceLogError::build_failed_write(e, path))?;
        info!("Successfully wrote maintenance log to {}.", path.display());
        Ok(())
    }

    pub fn ids(&self) -> Keys<'_, String, ServiceMetdata> {
        self.services.keys()
    }

    pub fn get(&self, id: &str) -> Option<&ServiceMetdata> {
        self.services.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut ServiceMetdata> {
        self.services.get_mut(id)
    }

    pub fn contains(&self, id: &str) -> bool {
        self.services.contains_key(id)
    }

    pub fn insert(&mut self, id: &str, metadata: ServiceMetdata) -> Option<ServiceMetdata> {
        self.services.insert(id.into(), metadata)
    }

    pub fn remove(&mut self, id: &str) -> Option<ServiceMetdata> {
        self.services.remove(id)
    }
}

fn build_subdirs(path: &Path) -> Result<(), MaintenanceLogError> {
    if let Some(parent) = path.parent() {
        debug!(
            "Creating subdirs for maintenance log at path {}.",
            parent.display()
        );
        std::fs::create_dir_all(parent)
            .map_err(|e| MaintenanceLogError::build_failed_write(e, path))?;
        debug!("Subdirs created.");
    }
    Ok(())
}
