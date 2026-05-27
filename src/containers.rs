use crate::cmds::init::ServiceInterval as InitServiceInterval;
use crate::cmds::init::{NextService, PreviousService};
use crate::errors::ContainersError;
use chrono::NaiveDate;
use log::{debug, info};
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

    pub fn write<P: AsRef<Path>>(&self, path: P) -> Result<(), ContainersError> {
        let path = path.as_ref();
        info!("Writing maintenance log to {}", path.display());
        let log_string = toml::to_string_pretty(&self)?;
        build_subdirs(path)?;
        std::fs::write(path, log_string)
            .map_err(|e| ContainersError::build_failed_write(e, path))?;
        info!("Successfully wrote maintenance log to {}", path.display());
        Ok(())
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

fn build_subdirs(path: &Path) -> Result<(), ContainersError> {
    if let Some(parent) = path.parent() {
        debug!(
            "Creating subdirs for maintenance log at path {}",
            parent.display()
        );
        std::fs::create_dir_all(parent)
            .map_err(|e| ContainersError::build_failed_write(e, path))?;
        debug!("Subdirs created");
    }
    Ok(())
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

impl fmt::Display for ServiceMetdata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Name: {}", self.name)?;
        writeln!(f, "Service Interval Miles: {}", self.service_interval.miles)?;
        writeln!(
            f,
            "Service Interval Months: {}",
            self.service_interval.months
        )?;
        writeln!(f, "Next Service Miles: {}", self.next_service.miles)?;
        write!(f, "Next Service Date: {}", self.next_service.date)?;

        match &self.previous_services {
            Some(previous_services) => {
                write!(f, "\nPrevious Services:")?;
                previous_services.iter().try_for_each(|service| {
                    write!(f, "\n  - {}/{} miles", service.date, service.miles)
                })?
            }
            None => write!(f, "\nPrevious Services: None")?,
        }

        if let Some(notes) = &self.notes {
            write!(f, "\nNotes:")?;
            notes
                .iter()
                .try_for_each(|note| write!(f, "\n  - {note}"))?
        }

        Ok(())
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

impl From<NextService> for ServiceEvent {
    fn from(value: NextService) -> Self {
        Self {
            miles: value.next_service_miles(),
            date: value.next_service_date(),
        }
    }
}

impl From<PreviousService> for ServiceEvent {
    fn from(value: PreviousService) -> Self {
        Self {
            miles: value.miles(),
            date: value.date(),
        }
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

impl From<InitServiceInterval> for ServiceInterval {
    fn from(value: InitServiceInterval) -> Self {
        Self {
            miles: value.miles_interval(),
            months: value.monthly_interval(),
        }
    }
}
