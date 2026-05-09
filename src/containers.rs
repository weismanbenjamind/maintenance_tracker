use std::collections::HashMap;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MaintenanceLog {
    services: HashMap<String, ServiceMetdata>,
}

impl MaintenanceLog {
    #[allow(dead_code)]
    fn new(services: HashMap<String, ServiceMetdata>) -> Self {
        Self { services }
    }

    #[allow(dead_code)]
    fn as_map(&self) -> &HashMap<String, ServiceMetdata> {
        &self.services
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ServiceMetdata {
    name: String,
    service_interval: ServiceInterval,
    next_service: ServiceEvent,
    previous_services: Vec<ServiceEvent>,
    notes: Vec<String>,
}

impl ServiceMetdata {
    #[allow(dead_code)]
    fn new(
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
    fn name(&self) -> &str {
        &self.name
    }

    #[allow(dead_code)]
    fn service_interval(&self) -> ServiceInterval {
        self.service_interval
    }

    #[allow(dead_code)]
    fn next_service(&self) -> ServiceEvent {
        self.next_service
    }

    #[allow(dead_code)]
    fn previous_services(&self) -> &[ServiceEvent] {
        &self.previous_services
    }

    #[allow(dead_code)]
    fn notes(&self) -> &[String] {
        &self.notes
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
struct ServiceEvent {
    miles: u32,
    date: NaiveDate,
}

impl ServiceEvent {
    #[allow(dead_code)]
    fn new(miles: u32, date: NaiveDate) -> Self {
        Self { miles, date }
    }

    #[allow(dead_code)]
    fn miles(&self) -> u32 {
        self.miles
    }

    #[allow(dead_code)]
    fn date(&self) -> NaiveDate {
        self.date
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
struct ServiceInterval {
    miles: u32,
    months: u32,
}

impl ServiceInterval {
    #[allow(dead_code)]
    fn new(miles: u32, months: u32) -> Self {
        Self { miles, months }
    }

    #[allow(dead_code)]
    fn miles(&self) -> u32 {
        self.miles
    }

    #[allow(dead_code)]
    fn months(&self) -> u32 {
        self.months
    }

    #[allow(dead_code)]
    fn years(&self) -> f64 {
        self.months as f64 / 12.0
    }
}
