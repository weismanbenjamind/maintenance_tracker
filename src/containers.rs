use crate::errors::ContainersError;
use chrono::NaiveDate;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::hash_map::Keys;
use std::fmt;
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

    pub fn get(&self, id: &str) -> Option<&ServiceMetdata> {
        self.services.get(id)
    }

    pub fn contains(&self, id: &str) -> bool {
        self.services.contains_key(id)
    }

    pub fn insert(&mut self, id: &str, metadata: ServiceMetdata) -> Option<ServiceMetdata> {
        self.services.insert(id.into(), metadata)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServiceMetdata {
    name: String,
    service_interval: ServiceInterval,
    next_service: ServiceEvent,
    previous_services: Option<Vec<ServiceEvent>>,
    notes: Option<Vec<String>>,
}

impl ServiceMetdata {
    #[allow(dead_code)]
    pub fn new(
        name: &str,
        service_interval: ServiceInterval,
        next_service: ServiceEvent,
        previous_services: Option<Vec<ServiceEvent>>,
        notes: Option<Vec<String>>,
    ) -> Self {
        Self {
            name: name.into(),
            service_interval,
            next_service,
            previous_services,
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
    pub fn previous_services(&self) -> Option<&[ServiceEvent]> {
        self.previous_services.as_deref()
    }

    #[allow(dead_code)]
    pub fn notes(&self) -> Option<&[String]> {
        self.notes.as_deref()
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

impl fmt::Display for ServiceEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Miles: {}", self.miles)?;
        write!(f, "Date: {}", self.date)
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
