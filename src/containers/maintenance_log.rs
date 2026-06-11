use crate::containers::ServiceMetdata;
use crate::errors::{IdNotFoundError, MaintenanceLogError};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::hash_map::{Keys, Values};
use std::path::Path;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MaintenanceLog {
    services: HashMap<String, ServiceMetdata>,
}

impl MaintenanceLog {
    pub(crate) fn load<P: AsRef<Path>>(path: P) -> Result<MaintenanceLog, MaintenanceLogError> {
        let path = path.as_ref();
        info!("Reading maintenance log path at {}.", path.display());
        let log = std::fs::read_to_string(path)
            .map_err(|source| MaintenanceLogError::FailedLoad(path.into(), source))?;
        let log = toml::from_str::<MaintenanceLog>(&log)?;
        info!("Successfully read maintenance log");
        Ok(log)
    }

    pub(crate) fn write<P: AsRef<Path>>(&self, path: P) -> Result<(), MaintenanceLogError> {
        let path = path.as_ref();
        info!("Writing maintenance log to {}.", path.display());
        let log_string = toml::to_string_pretty(&self)?;
        build_subdirs(path)?;
        std::fs::write(path, log_string)
            .map_err(|e| MaintenanceLogError::new_failed_write(e, path))?;
        info!("Successfully wrote maintenance log to {}.", path.display());
        Ok(())
    }

    pub(crate) fn ids(&self) -> Keys<'_, String, ServiceMetdata> {
        self.services.keys()
    }

    pub(crate) fn metadata(&self) -> Values<'_, String, ServiceMetdata> {
        self.services.values()
    }

    pub(crate) fn get(&self, id: &str) -> Result<&ServiceMetdata, IdNotFoundError> {
        self.services
            .get(id)
            .ok_or(IdNotFoundError::IdNotFound(id.into()))
    }

    pub(crate) fn get_mut(&mut self, id: &str) -> Result<&mut ServiceMetdata, IdNotFoundError> {
        self.services
            .get_mut(id)
            .ok_or(IdNotFoundError::IdNotFound(id.into()))
    }

    pub(crate) fn contains(&self, id: &str) -> bool {
        self.services.contains_key(id)
    }

    pub(crate) fn insert(&mut self, id: &str, metadata: ServiceMetdata) -> Option<ServiceMetdata> {
        self.services.insert(id.into(), metadata)
    }

    pub(crate) fn remove(&mut self, id: &str) -> Option<ServiceMetdata> {
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
            .map_err(|e| MaintenanceLogError::new_failed_write(e, path))?;
        debug!("Subdirs created.");
    }
    Ok(())
}
