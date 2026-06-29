//! # Testing
//!
//! Houses utilities specific for testing.
//! Nothing in this module should be public to this crate

#![cfg(test)]

use crate::containers::{MaintenanceLog, ServiceMetdata};

use std::collections::HashMap;

/// Module which houses constants for testing.
/// All testing factory functions use these constants
/// to build their objects.
pub(crate) mod constants {
    use crate::containers::{ServiceEvent, ServiceInterval};
    use chrono::NaiveDate;

    pub(crate) const NAME: &str = "name";
    pub(crate) const SERVICE_INTERVAL: ServiceInterval = ServiceInterval::new(4000, 5);
    pub(crate) const NEXT_SERVICE: ServiceEvent =
        ServiceEvent::new(75000, NaiveDate::from_ymd_opt(2026, 6, 5).unwrap());
    pub(crate) const PREVIOUS_SERVICE: ServiceEvent =
        ServiceEvent::new(71000, NaiveDate::from_ymd_opt(2026, 2, 20).unwrap());
    pub(crate) const NOTE: &str = "Note";
    pub(crate) const ID: &str = "id";
}

/// Builds service metadata from testing::constants for testing.
/// Built like the following:
/// ```
/// let previous_services = vec![testing::constants::PREVIOUS_SERVICE];
/// let notes = vec![testing::constants::NOTE.to_string()];
///
/// ServiceMetdata::new(
///     testing::constants::NAME.into(),
///     testing::constants::SERVICE_INTERVAL,
///     testing::constants::NEXT_SERVICE,
///     Some(previous_services),
///     Some(notes),
/// )
pub(crate) fn build_metadata() -> ServiceMetdata {
    let previous_services = vec![constants::PREVIOUS_SERVICE];
    let notes = vec![constants::NOTE.to_string()];

    ServiceMetdata::new(
        constants::NAME.into(),
        constants::SERVICE_INTERVAL,
        constants::NEXT_SERVICE,
        Some(previous_services),
        Some(notes),
    )
}

/// Builds a maintenance log from testing::constants for testing.
/// Constains one id: `testing::constants::ID` which maps to the
/// return of `testing::build_metadata()`.
pub(crate) fn build_log() -> MaintenanceLog {
    let metadata = build_metadata();
    let mut map: HashMap<String, ServiceMetdata> = HashMap::new();
    map.insert(constants::ID.into(), metadata);

    MaintenanceLog::new(map)
}
