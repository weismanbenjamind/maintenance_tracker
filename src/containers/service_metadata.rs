//! # Service Metadata
//!
//! Container for housing metadata about a given service.

use super::notes::Notes;
use crate::containers::previous_services::PreviousServices;
use crate::containers::services::{ServiceEvent, ServiceInterval};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Struct to house metadata about a given service.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub(crate) struct ServiceMetdata {
    name: String,
    service_interval: ServiceInterval,
    next_service: ServiceEvent,
    previous_services: PreviousServices,
    notes: Notes,
}

impl ServiceMetdata {
    /// Create a new ServiceMetadata object.
    pub(crate) fn new(
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
            previous_services: PreviousServices::new(previous_services),
            notes: Notes::new(notes),
        }
    }

    /// Get a borrow of the service's name.
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    /// Set the service's name.
    pub(crate) fn set_name(&mut self, name: &str) {
        self.name = name.into()
    }

    /// Get a copy of the service's ServiceInterval.
    pub(crate) fn service_interval(&self) -> ServiceInterval {
        self.service_interval
    }

    /// Get a mutable borrow of the service's ServiceInterval.
    pub(crate) fn service_interval_mut(&mut self) -> &mut ServiceInterval {
        &mut self.service_interval
    }

    /// Get a copy of the service's next service.
    pub(crate) fn next_service(&self) -> ServiceEvent {
        self.next_service
    }

    /// Get a mutable borrow of the service's next service.
    pub(crate) fn next_service_mut(&mut self) -> &mut ServiceEvent {
        &mut self.next_service
    }

    /// Gets a mutable borrow to the notes.
    pub(crate) fn notes_mut(&mut self) -> &mut Notes {
        &mut self.notes
    }

    /// Gets a mutable borrow to the previous services.
    pub(crate) fn prev_services_mut(&mut self) -> &mut PreviousServices {
        &mut self.previous_services
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
        writeln!(f, "Next Service Miles: {}", self.next_service.miles())?;
        write!(f, "Next Service Date: {}", self.next_service.date())?;

        match self.previous_services.service_events() {
            Some(previous_services) => {
                write!(f, "\nPrevious Services:")?;
                previous_services.iter().try_for_each(|service| {
                    write!(f, "\n  - {}/{} miles", service.date(), service.miles())
                })?
            }
            None => write!(f, "\nPrevious Services: None")?,
        }

        if let Some(notes) = self.notes.try_get() {
            write!(f, "\nNotes:")?;
            notes
                .iter()
                .try_for_each(|note| write!(f, "\n  - {note}"))?
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    const NAME: &str = "name";
    const SERVICE_INTERVAL: ServiceInterval = ServiceInterval::new(4000, 5);
    const NEXT_SERVICE: ServiceEvent =
        ServiceEvent::new(75000, NaiveDate::from_ymd_opt(2026, 6, 5).unwrap());
    const PREVIOUS_SERVICE: ServiceEvent =
        ServiceEvent::new(71000, NaiveDate::from_ymd_opt(2026, 2, 20).unwrap());
    const NOTE: &str = "Note";

    fn build_service_metadata() -> ServiceMetdata {
        let previous_services = vec![PREVIOUS_SERVICE];
        let notes = vec![NOTE.to_string()];

        ServiceMetdata {
            name: NAME.into(),
            service_interval: SERVICE_INTERVAL,
            next_service: NEXT_SERVICE,
            previous_services: PreviousServices::new(Some(previous_services)),
            notes: Notes::new(Some(notes)),
        }
    }

    #[test]
    fn service_metadata_new() {
        let name = "name";
        let service_interval = ServiceInterval::new(4000, 5);
        let next_service = ServiceEvent::new(75000, NaiveDate::from_ymd_opt(2026, 6, 5).unwrap());
        let previous_services = vec![ServiceEvent::new(
            71000,
            NaiveDate::from_ymd_opt(2026, 2, 20).unwrap(),
        )];
        let notes = vec!["Note".to_string()];

        let service_metadata = ServiceMetdata::new(
            name,
            service_interval,
            next_service,
            Some(previous_services.clone()),
            Some(notes.clone()),
        );

        assert_eq!(service_metadata.name, name.to_string());
        assert_eq!(service_metadata.service_interval, service_interval);
        assert_eq!(service_metadata.next_service, next_service);
        assert_eq!(
            service_metadata.previous_services,
            PreviousServices::new(Some(previous_services))
        );
        assert_eq!(service_metadata.notes, Notes::new(Some(notes)));
    }

    #[test]
    fn service_metadata_getters() {
        let mut service_metadata = build_service_metadata();
        assert_eq!(service_metadata.name(), NAME);
        assert_eq!(service_metadata.service_interval(), SERVICE_INTERVAL);
        assert_eq!(*service_metadata.service_interval_mut(), SERVICE_INTERVAL);
        assert_eq!(service_metadata.next_service(), NEXT_SERVICE);
        assert_eq!(*service_metadata.next_service_mut(), NEXT_SERVICE);
        assert_eq!(
            *service_metadata.notes_mut(),
            Notes::new(Some(vec![NOTE.to_string()]))
        );
        assert_eq!(
            *service_metadata.prev_services_mut(),
            PreviousServices::new(Some(vec![PREVIOUS_SERVICE]))
        );
    }

    #[test]
    fn service_metadata_setters() {
        let mut service_metadata = build_service_metadata();
        let new_name = "new_name";
        assert_ne!(new_name, NAME);
        service_metadata.set_name(new_name);
        assert_eq!(service_metadata.name, new_name.to_string());
    }

    #[test]
    fn service_metadata_display_prev_services_and_notes_set() {
        let metadata = build_service_metadata();
        let found = format!("{}", metadata);
        let expected = format!(
            "Name: {}\n\
            Service Interval Miles: {}\n\
            Service Interval Months: {}\n\
            Next Service Miles: {}\n\
            Next Service Date: {}\n\
            Previous Services:\n  - {}/{} miles\n\
            Notes:\n  - {}",
            NAME,
            SERVICE_INTERVAL.miles,
            SERVICE_INTERVAL.months,
            NEXT_SERVICE.miles(),
            NEXT_SERVICE.date(),
            PREVIOUS_SERVICE.date(),
            PREVIOUS_SERVICE.miles(),
            NOTE,
        );
        assert_eq!(found, expected);
    }

    #[test]
    fn service_metadata_display_prev_services_and_notes_unset() {
        let metadata = ServiceMetdata {
            name: NAME.into(),
            service_interval: SERVICE_INTERVAL,
            next_service: NEXT_SERVICE,
            previous_services: PreviousServices::new(None),
            notes: Notes::new(None),
        };

        let found = format!("{}", metadata);
        let expected = format!(
            "Name: {}\n\
            Service Interval Miles: {}\n\
            Service Interval Months: {}\n\
            Next Service Miles: {}\n\
            Next Service Date: {}\n\
            Previous Services: None",
            NAME,
            SERVICE_INTERVAL.miles,
            SERVICE_INTERVAL.months,
            NEXT_SERVICE.miles(),
            NEXT_SERVICE.date(),
        );
        assert_eq!(found, expected);
    }
}
