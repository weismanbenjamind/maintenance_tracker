//! # Services
//!
//! Houses various containers relating to services

use crate::cmds::ServiceInterval as InitServiceInterval;
use crate::cmds::{NextService, PreviousService};
use crate::dates::months_to_days_floored;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Struct to represent a service event (e.g. miles on vehicle and date of event).
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub(crate) struct ServiceEvent {
    miles: u32,
    date: NaiveDate,
}

impl ServiceEvent {
    /// Create a new `ServiceEvent` witht the given `miles` and `date`.
    pub(crate) const fn new(miles: u32, date: NaiveDate) -> Self {
        Self { miles, date }
    }

    /// Get the miles of the `ServiceEvent`.
    /// Returns a copy of the actual miles the object houses.
    pub(crate) fn miles(&self) -> u32 {
        self.miles
    }

    /// Set the miles of `ServiceEvent`.
    pub(crate) fn set_miles(&mut self, miles: u32) {
        self.miles = miles
    }

    /// Get the date of the `ServiceEvent`.
    /// Returns a copy of the actual date the object houses.
    pub(crate) fn date(&self) -> NaiveDate {
        self.date
    }

    /// Set the date of the `ServiceEvent`.
    pub(crate) fn set_date(&mut self, date: NaiveDate) {
        self.date = date
    }

    /// Update the miles and/or date of the `ServiceEvent` or neither.
    /// Updates miles if `Some(miles)` is passed.
    /// Updates date if `Some(date)` is passed.
    /// Updates miles and date if `Some(miles)` and `Some(date)` are passed.
    /// Updates neither if `None` and `None` are passed.
    pub(crate) fn update(&mut self, miles: Option<u32>, date: Option<NaiveDate>) {
        if let Some(miles) = miles {
            self.miles = miles
        };

        if let Some(date) = date {
            self.date = date
        };
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

/// Struct to represent a service interval.
/// E.g. miles and months between services.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub(crate) struct ServiceInterval {
    pub(super) miles: u32,
    pub(super) months: u32,
}

impl ServiceInterval {
    /// Create a new `ServiceInterval`.
    pub(crate) const fn new(miles: u32, months: u32) -> Self {
        Self { miles, months }
    }

    /// Set the miles of the `ServiceInterval`.
    pub(crate) fn set_miles(&mut self, miles: u32) {
        self.miles = miles
    }

    /// Set the months of the `ServiceInterval`.
    pub(crate) fn set_months(&mut self, months: u32) {
        self.months = months
    }

    /// Get a copy of the miles on the `ServiceInterval`.
    pub(crate) fn miles(&self) -> u32 {
        self.miles
    }

    /// Get the days between services.
    /// Will floor the result when converting to days.
    /// Ensures never overshoot on maintenance.
    pub(crate) fn days(&self) -> i64 {
        months_to_days_floored(self.months)
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

#[cfg(test)]
mod tests {
    use super::*;

    const SERVICE_EVENT_MILES: u32 = 70000;
    const SERVICE_EVENT_DATE: NaiveDate = chrono::NaiveDate::from_ymd_opt(2026, 6, 2).unwrap();
    const SERVICE_INTERVAL_MILES: u32 = 4000;
    const SERVICE_INTERVAL_MONTHS: u32 = 5;

    fn build_service_event() -> ServiceEvent {
        ServiceEvent {
            miles: SERVICE_EVENT_MILES,
            date: SERVICE_EVENT_DATE,
        }
    }

    fn build_service_interval() -> ServiceInterval {
        ServiceInterval {
            miles: SERVICE_EVENT_MILES,
            months: SERVICE_INTERVAL_MONTHS,
        }
    }

    #[test]
    fn test_services_new_service_event() {
        let new = ServiceEvent::new(SERVICE_EVENT_MILES, SERVICE_EVENT_DATE);
        assert_eq!(new.miles, SERVICE_EVENT_MILES);
        assert_eq!(new.date, SERVICE_EVENT_DATE);
    }

    #[test]
    fn test_services_service_event_getters() {
        let service_event = build_service_event();

        assert_eq!(service_event.miles(), SERVICE_EVENT_MILES);
        assert_eq!(service_event.date(), SERVICE_EVENT_DATE);
    }

    #[test]
    fn test_services_service_event_setters() {
        let mut service_event = build_service_event();

        let new_miles: u32 = 100000;
        assert_ne!(service_event.miles, new_miles);
        service_event.set_miles(new_miles);
        assert_eq!(service_event.miles, new_miles);

        let new_date = chrono::NaiveDate::from_ymd_opt(2027, 8, 13).unwrap();
        assert_ne!(service_event.date, new_date);
        service_event.set_date(new_date);
        assert_eq!(service_event.date, new_date);
    }

    #[test]
    fn test_services_service_event_update_miles() {
        let mut service_event = build_service_event();

        let new_miles: u32 = 100000;
        assert_ne!(service_event.miles, new_miles);
        service_event.update(Some(new_miles), None);
        assert_eq!(service_event.miles, new_miles);
        assert_eq!(service_event.date, SERVICE_EVENT_DATE);
    }

    #[test]
    fn test_services_service_event_update_date() {
        let mut service_event = build_service_event();

        let new_date = chrono::NaiveDate::from_ymd_opt(2027, 8, 13).unwrap();
        assert_ne!(service_event.date, new_date);
        service_event.update(None, Some(new_date));
        assert_eq!(service_event.date, new_date);
        assert_eq!(service_event.miles, SERVICE_EVENT_MILES);
    }

    #[test]
    fn test_services_service_event_update_miles_and_date() {
        let mut service_event = build_service_event();

        let new_miles: u32 = 100000;
        assert_ne!(service_event.miles, new_miles);

        let new_date = chrono::NaiveDate::from_ymd_opt(2027, 8, 13).unwrap();
        assert_ne!(service_event.date, new_date);

        service_event.update(Some(new_miles), Some(new_date));
        assert_eq!(service_event.miles, new_miles);
        assert_eq!(service_event.date, new_date);
    }

    #[test]
    fn test_services_service_event_update_nothing() {
        let mut service_event = build_service_event();

        service_event.update(None, None);
        assert_eq!(service_event.miles, SERVICE_EVENT_MILES);
        assert_eq!(service_event.date, SERVICE_EVENT_DATE);
    }

    #[test]
    fn test_services_service_event_fmt() {
        let service_event = build_service_event();
        let found = format!("{service_event}");
        let expected = format!(
            "Miles: {}\nDate: {}",
            service_event.miles, service_event.date
        );
        assert_eq!(found, expected);
    }

    #[test]
    fn test_services_service_event_from_next_service() {
        let next_service = NextService::new(SERVICE_EVENT_MILES, SERVICE_EVENT_DATE);
        let found = ServiceEvent::from(next_service);
        assert_eq!(found.miles, SERVICE_EVENT_MILES);
        assert_eq!(found.date, SERVICE_EVENT_DATE);
    }

    #[test]
    fn test_services_service_event_from_previous_service() {
        let prev_service = PreviousService::new(SERVICE_EVENT_MILES, SERVICE_EVENT_DATE);
        let found = ServiceEvent::from(prev_service);
        assert_eq!(found.miles, SERVICE_EVENT_MILES);
        assert_eq!(found.date, SERVICE_EVENT_DATE);
    }

    #[test]
    fn services_service_interval_new() {
        let service_interval =
            ServiceInterval::new(SERVICE_INTERVAL_MILES, SERVICE_INTERVAL_MONTHS);
        assert_eq!(service_interval.miles, SERVICE_INTERVAL_MILES);
        assert_eq!(service_interval.months, SERVICE_INTERVAL_MONTHS);
    }

    #[test]
    fn services_service_interval_getters() {
        let service_interval = build_service_interval();
        assert_eq!(service_interval.miles(), SERVICE_EVENT_MILES);
    }

    #[test]
    fn services_service_interval_setters() {
        let mut service_interval = build_service_interval();

        let new_miles: u32 = 100000;
        assert_ne!(service_interval.miles, new_miles);

        let new_months: u32 = 6;
        assert_ne!(service_interval.months, new_months);

        service_interval.set_miles(new_miles);
        assert_eq!(service_interval.miles, new_miles);

        service_interval.set_months(new_months);
        assert_eq!(service_interval.months, new_months);
    }

    #[test]
    fn services_service_interval_days_no_round() {
        let service_interval = ServiceInterval {
            miles: SERVICE_EVENT_MILES,
            months: 12,
        };
        assert_eq!(service_interval.days(), 365);
    }

    #[test]
    fn services_service_interval_days_round() {
        let service_interval = ServiceInterval {
            miles: SERVICE_EVENT_MILES,
            months: 13,
        };
        assert_eq!(service_interval.days(), 395);
    }

    #[test]
    fn services_service_interval_from_init_service_interval() {
        let init_service_interval =
            InitServiceInterval::new(SERVICE_EVENT_MILES, SERVICE_INTERVAL_MONTHS);
        let found = ServiceInterval::from(init_service_interval);
        assert_eq!(found.miles, SERVICE_EVENT_MILES);
        assert_eq!(found.months, SERVICE_INTERVAL_MONTHS);
    }
}
