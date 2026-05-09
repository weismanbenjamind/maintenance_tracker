use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServiceMetdata {
    name: String,
    service_interval: ServiceInterval,
    next_service: ServiceInterval,
    previous_services: Vec<ServiceInterval>,
    notes: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
struct ServiceInterval {
    miles: u32,
    date: NaiveDate,
}
