use crate::errors::ContainersError;
use chrono::NaiveDate;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::hash_map::Keys;
use std::path::Path;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MaintenanceLog {
    services: HashMap<String, ServiceMetdata>,
}

impl MaintenanceLog {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<MaintenanceLog, ContainersError> {
        let path = path.as_ref();
        info!("Reading maintenance log path at {}.", path.display());
        let log = std::fs::read_to_string(path).map_err(|source| ContainersError::FailedLoad {
            path: path.into(),
            source,
        })?;
        Ok(toml::from_str::<MaintenanceLog>(&log)?)
    }

    #[allow(dead_code)]
    pub fn new(services: HashMap<String, ServiceMetdata>) -> Self {
        Self { services }
    }

    #[allow(dead_code)]
    pub fn as_map(&self) -> &HashMap<String, ServiceMetdata> {
        &self.services
    }

    pub fn ids(&self) -> Keys<'_, String, ServiceMetdata> {
        self.services.keys()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServiceMetdata {
    name: String,
    service_interval: ServiceInterval,
    next_service: ServiceEvent,
    previous_services: Vec<ServiceEvent>,
    notes: Vec<String>,
}

impl ServiceMetdata {
    #[allow(dead_code)]
    pub fn new(
        name: &str,
        service_interval: ServiceInterval,
        next_service: ServiceEvent,
        previous_services: &[ServiceEvent], // Since ServiceEvent is clone can pass a slice. Only one heap allocation for the Clone
        notes: Vec<String>, // Don't pass slice here. Will cause a Vec heap allocation along with all heap allocations for Strings
    ) -> Self {
        Self {
            name: name.into(),
            service_interval,
            next_service,
            previous_services: previous_services.into(),
            notes,
        }
    }
    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[allow(dead_code)]
    pub fn service_interval(&self) -> ServiceInterval {
        self.service_interval
    }

    #[allow(dead_code)]
    pub fn next_service(&self) -> ServiceEvent {
        self.next_service
    }

    #[allow(dead_code)]
    pub fn previous_services(&self) -> &[ServiceEvent] {
        &self.previous_services
    }

    #[allow(dead_code)]
    pub fn notes(&self) -> &[String] {
        &self.notes
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct ServiceEvent {
    miles: u32,
    date: NaiveDate,
}

impl ServiceEvent {
    #[allow(dead_code)]
    pub fn new(miles: u32, date: NaiveDate) -> Self {
        Self { miles, date }
    }

    #[allow(dead_code)]
    pub fn miles(&self) -> u32 {
        self.miles
    }

    #[allow(dead_code)]
    pub fn date(&self) -> NaiveDate {
        self.date
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct ServiceInterval {
    miles: u32,
    months: u32,
}

impl ServiceInterval {
    #[allow(dead_code)]
    pub fn new(miles: u32, months: u32) -> Self {
        Self { miles, months }
    }

    #[allow(dead_code)]
    pub fn miles(&self) -> u32 {
        self.miles
    }

    #[allow(dead_code)]
    pub fn months(&self) -> u32 {
        self.months
    }

    #[allow(dead_code)]
    pub fn years(&self) -> f64 {
        self.months as f64 / 12.0
    }
}
