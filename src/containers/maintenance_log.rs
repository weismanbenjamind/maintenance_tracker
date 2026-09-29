//! # Maintenance Log
//!
//! Container to model entire maintenance log

use std::{
    collections::{HashMap, hash_map::Keys},
    path::Path,
};

use log::{debug, info};
use serde::{Deserialize, Serialize};

use crate::{
    containers::ServiceMetdata,
    errors::{IdNotFoundError, MaintenanceLogError},
};

/// Struct to model maintenance log.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MaintenanceLog {
    services: HashMap<String, ServiceMetdata>,
}

impl MaintenanceLog {
    /// Create a new MaintenanceLog.
    pub(crate) fn new(services: HashMap<String, ServiceMetdata>) -> Self {
        Self { services }
    }

    /// Load the MaintenanceLog from a file
    pub(crate) fn load<P: AsRef<Path>>(path: P) -> Result<MaintenanceLog, MaintenanceLogError> {
        let path = path.as_ref();
        info!("Reading maintenance log path at {}.", path.display());
        let log = std::fs::read_to_string(path)
            .map_err(|source| MaintenanceLogError::FailedLoad(path.into(), source))?;
        let log = toml::from_str::<MaintenanceLog>(&log)?;
        info!("Successfully read maintenance log");
        Ok(log)
    }

    /// Write the MaintenanceLog to a file.
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

    /// Get the ids (unique identifiers) of the log.
    pub(crate) fn ids(&self) -> Keys<'_, String, ServiceMetdata> {
        self.services.keys()
    }

    /// Get a vector of metadata sorted by the name of the service.
    pub(crate) fn metadata_sorted(&self) -> Vec<&ServiceMetdata> {
        let mut metadata = self.services.values().collect::<Vec<&ServiceMetdata>>();
        metadata.sort_by_key(|a| a.name().to_lowercase());
        metadata
    }

    /// Get a borrow of the metadata for a given id.
    pub(crate) fn get(&self, id: &str) -> Result<&ServiceMetdata, IdNotFoundError> {
        self.services
            .get(id)
            .ok_or(IdNotFoundError::IdNotFound(id.into()))
    }

    /// Get a mutable borrow of the metadata for a given id.
    pub(crate) fn get_mut(&mut self, id: &str) -> Result<&mut ServiceMetdata, IdNotFoundError> {
        self.services
            .get_mut(id)
            .ok_or(IdNotFoundError::IdNotFound(id.into()))
    }

    /// Get a bool indicating if the id is present in the maintenance log.
    pub(crate) fn contains(&self, id: &str) -> bool {
        self.services.contains_key(id)
    }

    /// Insert a new a new metadata field for the given id into the log.
    /// Will return `Some(metadata)` if the id aleady exists.
    /// Will return `None` if the id does not exist.
    pub(crate) fn insert(&mut self, id: &str, metadata: ServiceMetdata) -> Option<ServiceMetdata> {
        self.services.insert(id.into(), metadata)
    }

    /// Remove a metadata field from the log for the given id.
    /// Will return `Some(metadata)` if the field exists and was remove.
    /// Will return `None` otherwise.
    pub(crate) fn remove(&mut self, id: &str) -> Option<ServiceMetdata> {
        self.services.remove(id)
    }
}

/// Build the subdirs leading up to a path.
/// Will not build the final path element.
/// For example `build_subdirs("/path/to/a/file")` will build `/path/to/a`,
/// If the parent dir to the final path element exists, no action will be taken.
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
    use std::{io::Write, path::PathBuf};

    use indoc::indoc;
    use tempfile::{NamedTempFile, TempDir};

    use super::*;
    use crate::testing::{build_log, build_metadata, constants::ID};

    #[test]
    fn test_maintenance_log_constructor() {
        let mut map: HashMap<String, ServiceMetdata> = HashMap::new();
        map.insert(ID.into(), build_metadata());
        let log = MaintenanceLog::new(map.clone());
        assert_eq!(log.services, map);
    }

    #[test]
    fn test_maintenance_log_write_ok() {
        let log = build_log();
        let tempfile = NamedTempFile::with_suffix(".toml").unwrap();
        log.write(&tempfile).unwrap();
        let as_string = std::fs::read_to_string(tempfile).unwrap();
        assert!(as_string.contains("[services.id]"));
    }

    #[test]
    fn test_maintenance_log_load_file_not_exist() {
        let path = PathBuf::from("/should/not/exist.toml");
        assert!(!path.exists());
        let err = MaintenanceLog::load(&path).unwrap_err();
        match err {
            MaintenanceLogError::FailedLoad(_, _) => (),
            _ => panic!("Expected MaintenanceLogError::FailedLoad"),
        }
    }

    #[test]
    fn test_maintenance_log_load_invalid_toml() {
        // Missing service interval
        let log_str = indoc! {"
            [services.transmission_fluid]
            name = \"Transmission Fluid\"

            [services.transmission_fluid.next_service]
            miles = 100000
            date = \"2028-05-01\"
        "};

        let mut tmp_file = NamedTempFile::with_suffix(".toml").unwrap();
        tmp_file.write(log_str.as_bytes()).unwrap();

        let err = MaintenanceLog::load(&tmp_file).unwrap_err();

        match err {
            MaintenanceLogError::FailedDerserialize(_) => (),
            _ => panic!("Expected MaintenanceLogError::FailedDerserialize"),
        }
    }

    #[test]
    fn test_maintenance_log_load_ok() {
        let log_str = indoc! {"
            [services.transmission_fluid]
            name = \"Transmission Fluid\"
            notes = [
                \"Doing 29000 mile interval\",
                \"Doing 17 month interval\",
            ]

            [services.transmission_fluid.service_interval]
            miles = 29000
            months = 35

            [services.transmission_fluid.next_service]
            miles = 100000
            date = \"2028-05-01\"

            [[services.transmission_fluid.previous_services]]
            miles = 70000
            date = \"2025-06-01\"
        "};

        let mut tmp_file = NamedTempFile::with_suffix(".toml").unwrap();
        tmp_file.write(log_str.as_bytes()).unwrap();

        let found = MaintenanceLog::load(&tmp_file).unwrap();
        let map = found.services;
        assert_eq!(map.len(), 1);

        let target_key = "transmission_fluid";
        assert!(map.contains_key(target_key));
        assert_eq!(map.get(target_key).unwrap().name(), "Transmission Fluid");
    }

    #[test]
    fn maintenance_log_ids() {
        let mut log = build_log();
        let key = "new_key";
        assert_ne!(ID, key);
        log.services.insert(key.into(), build_metadata());

        let mut expected = vec![ID, key];
        expected.sort();

        let mut found = log.ids().collect::<Vec<&String>>();
        found.sort();

        assert_eq!(expected, found);
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

    #[test]
    fn maintenance_log_create_subdirs_create() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        let to_build = path.join("subdir").join("log.toml");
        assert!(!to_build.exists());
        build_subdirs(&to_build).unwrap();
        assert!(to_build.parent().unwrap().exists());
    }

    #[test]
    fn maintenance_log_create_subdirs_no_create() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        let count_pre_build = path.read_dir().unwrap().count();

        let to_build = path.join("log.toml");
        build_subdirs(&to_build).unwrap();

        let count_post_build = path.read_dir().unwrap().count();
        assert_eq!(count_pre_build, count_post_build);
    }
}
