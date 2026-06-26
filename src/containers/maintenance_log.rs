// TODO - WRITE TESTS

use crate::containers::ServiceMetdata;
use crate::errors::{IdNotFoundError, MaintenanceLogError};
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
    pub(crate) fn new(services: HashMap<String, ServiceMetdata>) -> Self {
        Self { services }
    }

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

    pub(crate) fn metadata_sorted(&self) -> Vec<&ServiceMetdata> {
        let mut metadata = self.services.values().collect::<Vec<&ServiceMetdata>>();
        metadata.sort_by_key(|a| a.name().to_lowercase());
        metadata
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::containers::{ServiceEvent, ServiceInterval};

    use chrono::NaiveDate;

    const NAME: &str = "name";
    const SERVICE_INTERVAL: ServiceInterval = ServiceInterval::new(4000, 5);
    const NEXT_SERVICE: ServiceEvent =
        ServiceEvent::new(75000, NaiveDate::from_ymd_opt(2026, 6, 5).unwrap());
    const PREVIOUS_SERVICE: ServiceEvent =
        ServiceEvent::new(71000, NaiveDate::from_ymd_opt(2026, 2, 20).unwrap());
    const NOTE: &str = "Note";
    const ID: &str = "id";

    fn build_metadata() -> ServiceMetdata {
        let previous_services = vec![PREVIOUS_SERVICE];
        let notes = vec![NOTE.to_string()];

        ServiceMetdata::new(
            NAME.into(),
            SERVICE_INTERVAL,
            NEXT_SERVICE,
            Some(previous_services),
            Some(notes),
        )
    }

    fn build_log() -> MaintenanceLog {
        let metadata = build_metadata();
        let mut map: HashMap<String, ServiceMetdata> = HashMap::new();
        map.insert(ID.into(), metadata);

        MaintenanceLog { services: map }
    }

    #[test]
    fn maintenance_log_ids() {
        todo!("Pick up testing here")
    }

    #[test]
    fn maintenance_log_metadata_sorted_empty_map() {
        let mut log = build_log();
        log.services.clear();

        let sorted = log.metadata_sorted();
        assert!(sorted.is_empty());
    }

    #[test]
    fn maintenance_log_metadata_sorted() {
        let mut metadata = build_metadata();
        let new_name = "a_name";
        metadata.set_name(new_name);

        let mut log = build_log();
        let new_id = "new_id";
        assert_ne!(new_id, ID);

        log.insert(new_id, metadata.clone());

        let sorted = log.metadata_sorted();
        assert_eq!(sorted.len(), 2);
        assert_eq!(*sorted[0], metadata);
        assert_eq!(*sorted[1], build_metadata());
    }

    #[test]
    fn maintenance_log_get_found() {
        assert_eq!(*build_log().get(ID).unwrap(), build_metadata());
    }

    #[test]
    fn maintenance_log_get_not_found() {
        assert!(build_log().get("some_missing_id").is_err());
    }

    #[test]
    fn maintenance_log_get_mut_found() {
        let mut log = build_log();
        assert_eq!(*log.get_mut(ID).unwrap(), build_metadata());
    }

    #[test]
    fn maintenance_log_get_mut_not_found() {
        let mut log = build_log();
        assert!(log.get_mut("some_missing_id").is_err());
    }

    #[test]
    fn maintenance_log_contains() {
        let log = build_log();
        assert!(log.contains(ID));
        assert!(!log.contains("some_other_id"));
    }

    #[test]
    fn maintenance_log_insert_present_item() {
        let mut metadata = build_metadata();
        let updated_name = "new_name";
        metadata.set_name(updated_name);

        let mut log = build_log();
        let result = log.insert(ID, metadata);
        assert_eq!(result.unwrap(), build_metadata());

        let services = log.services;
        assert_eq!(services.len(), 1);
        let found = services.get(ID).unwrap();
        assert_eq!(found.name(), updated_name);
    }

    #[test]
    fn maintenance_log_insert_new_item() {
        let mut metadata = build_metadata();
        let updated_name = "new_name";
        metadata.set_name(updated_name);
        let new_id = "new_item";

        let mut log = build_log();
        let result = log.insert(new_id, metadata);
        assert!(result.is_none());

        let services = log.services;
        assert_eq!(services.len(), 2);
        let found = services.get(new_id).unwrap();
        assert_eq!(found.name(), updated_name);
    }

    #[test]
    fn maintenance_log_remove_id_found() {
        let mut log = build_log();
        assert_eq!(log.remove(ID).unwrap(), build_metadata());
    }

    #[test]
    fn maintenance_log_remove_id_not_found() {
        let mut log = build_log();
        let id_to_remove = "not_found";
        assert_ne!(ID, id_to_remove);
        assert!(log.remove(id_to_remove).is_none());
    }
}
