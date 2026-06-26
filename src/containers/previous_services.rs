//! # Previous Services
//!
//! Houses functionality for dealing with previous services of a maintenance item

use super::services::ServiceEvent;
use crate::errors::PreviousServicesError;
use chrono::NaiveDate;
use log::{debug, info};
use serde::{Deserialize, Serialize};

/// Struct to house previous services.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(transparent)]
pub(crate) struct PreviousServices {
    service_events: Option<Vec<ServiceEvent>>,
}

impl PreviousServices {
    /// Create a new previous service object.
    pub(crate) fn new(service_events: Option<Vec<ServiceEvent>>) -> Self {
        Self { service_events }
    }

    /// Add a completed service to the previous services.
    pub(crate) fn add(&mut self, miles: u32, date: NaiveDate) {
        let service_event = ServiceEvent::new(miles, date);
        match &mut self.service_events {
            Some(service_events) => service_events.push(service_event),
            None => self.service_events = Some(vec![service_event]),
        }
    }

    /// Get all previous services if they exist. Returns None if no previous services exist.
    pub(crate) fn service_events(&self) -> Option<&[ServiceEvent]> {
        self.service_events.as_deref()
    }

    /// Get a mutable borrow to all previous services if they exist. Returns None if no previous services exist.
    fn get_previous_services_mut(
        &mut self,
    ) -> Result<&mut Vec<ServiceEvent>, PreviousServicesError> {
        match &mut self.service_events {
            Some(service_events) => Ok(service_events),
            None => Err(PreviousServicesError::PreviousServicesNotSet),
        }
    }

    /// Replace a previous service's miles, date, or both
    /// targetting a specific miles, date, or miles/date combination.
    pub(crate) fn replace(
        &mut self,
        curr_miles: Option<u32>,
        curr_date: Option<NaiveDate>,
        new_miles: Option<u32>,
        new_date: Option<NaiveDate>,
    ) -> Result<(), PreviousServicesError> {
        info!("Attempting to replace previous service");
        let prev_services = self.get_previous_services_mut()?;
        let idx = get_prev_service_idx(prev_services, curr_miles, curr_date)?;

        // Note - .get_mut() should never error (return None) because get_prev_service_idx ensures we have the service event
        // Leaving here as a fallback
        prev_services
            .get_mut(idx)
            .ok_or(PreviousServicesError::ServiceEventNotFound)?
            .update(new_miles, new_date);

        info!("Replaced previous service");

        Ok(())
    }

    /// Removes a previous service for the given miles, date, or miles/date combination.
    /// Must pass at least one of miles or date.
    pub(crate) fn remove(
        &mut self,
        miles: Option<u32>,
        date: Option<NaiveDate>,
    ) -> Result<(), PreviousServicesError> {
        info!("Attempting to remove service event");

        let prev_services = self.get_previous_services_mut()?;
        let idx = get_prev_service_idx(prev_services, miles, date)?;

        // remove can panic
        // get_prev_service_idx should ensure this index exists so the removal call is safe
        prev_services.remove(idx);

        info!("Removed service event");

        Ok(())
    }

    pub(crate) fn clear(&mut self) {
        self.service_events = None
    }
}

/// Given a slice of service events,
/// Finds the index which equals the miles, date, or miles/date combination.
/// Will return Err if not exactly one service event is found.
/// Must pass one of miles, date, or both.
fn get_prev_service_idx(
    prev_services: &[ServiceEvent],
    miles: Option<u32>,
    date: Option<NaiveDate>,
) -> Result<usize, PreviousServicesError> {
    let idxs = filter_prev_services_idxs(prev_services, miles, date)?;
    if idxs.is_empty() {
        debug!("No previous events found for provided filter.");
        Err(PreviousServicesError::ServiceEventNotFound)
    } else if idxs.len() > 1 {
        debug!("Found multiple previous service events for provided filter.");
        Err(PreviousServicesError::MultipleServiceEvents)
    } else {
        debug!("Found service event.");
        Ok(idxs[0])
    }
}

/// Given a slice of service events,
/// Finds the indices which equals the miles, date, or miles/date combination.
/// Must pass one of miles, date, or both.
fn filter_prev_services_idxs(
    prev_services: &[ServiceEvent],
    miles: Option<u32>,
    date: Option<NaiveDate>,
) -> Result<Vec<usize>, PreviousServicesError> {
    let result: Vec<usize> = match (miles, date) {
        (None, None) => {
            return Err(PreviousServicesError::InvalidOptionalArgs);
        }
        (Some(miles), None) => prev_services
            .iter()
            .enumerate()
            .filter(|(_idx, service_event)| service_event.miles() == miles)
            .map(|(idx, _service_event)| idx)
            .collect(),
        (None, Some(date)) => prev_services
            .iter()
            .enumerate()
            .filter(|(_idx, service_event)| service_event.date() == date)
            .map(|(idx, _service_event)| idx)
            .collect(),
        (Some(miles), Some(date)) => prev_services
            .iter()
            .enumerate()
            .filter(|(_idx, service_event)| {
                service_event.miles() == miles && service_event.date() == date
            })
            .map(|(idx, _service_event)| idx)
            .collect(),
    };

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    const MILES: ServiceEvent =
        ServiceEvent::new(70000, NaiveDate::from_ymd_opt(2026, 6, 1).unwrap());
    const DATE: ServiceEvent =
        ServiceEvent::new(74000, NaiveDate::from_ymd_opt(2026, 10, 14).unwrap());
    const MILES_DATE: ServiceEvent =
        ServiceEvent::new(77000, NaiveDate::from_ymd_opt(2027, 2, 14).unwrap());
    const OVERLAP: ServiceEvent = ServiceEvent::new(MILES_DATE.miles(), MILES_DATE.date());
    const PREV_SERVICES: [ServiceEvent; 4] = [MILES, DATE, MILES_DATE, OVERLAP];

    fn build_previous_services() -> PreviousServices {
        PreviousServices {
            service_events: Some(PREV_SERVICES.into()),
        }
    }

    #[test]
    fn previous_services_previous_services_add() {
        todo!("Start with this test")
    }

    #[test]
    fn previous_services_previous_services_clear() {
        let mut prev_services = build_previous_services();
        assert!(prev_services.service_events.is_some());
        prev_services.clear();
        assert!(prev_services.service_events.is_none());
    }

    #[test]
    fn previous_services_previous_services_constructor_some() {
        let found = PreviousServices::new(Some(PREV_SERVICES.into()));
        assert_eq!(found.service_events.unwrap(), PREV_SERVICES);
    }

    #[test]
    fn previous_services_previous_services_constructor_none() {
        let found = PreviousServices::new(None);
        assert!(found.service_events.is_none());
    }

    #[test]
    fn previous_services_get_prev_service_idx_miles() {
        let found = get_prev_service_idx(&PREV_SERVICES, Some(MILES.miles()), None).unwrap();
        assert_eq!(found, 0);
    }

    #[test]
    fn previous_services_get_prev_service_idx_date() {
        let found = get_prev_service_idx(&PREV_SERVICES, None, Some(DATE.date())).unwrap();
        assert_eq!(found, 1);
    }

    #[test]
    fn previous_services_get_prev_service_idx_miles_date() {
        let found =
            get_prev_service_idx(&PREV_SERVICES, Some(MILES.miles()), Some(MILES.date())).unwrap();
        assert_eq!(found, 0);
    }

    #[test]
    fn previous_services_get_prev_service_idx_invalid_args() {
        let found = get_prev_service_idx(&PREV_SERVICES, None, None).unwrap_err();
        match found {
            PreviousServicesError::InvalidOptionalArgs => (),
            _ => panic!("Expected PreviousServicesError::InvalidOptionalArgs"),
        }
    }

    #[test]
    fn previous_services_get_prev_service_idx_multi() {
        let found =
            get_prev_service_idx(&PREV_SERVICES, Some(OVERLAP.miles()), Some(OVERLAP.date()))
                .unwrap_err();

        match found {
            PreviousServicesError::MultipleServiceEvents => (),
            _ => panic!("Expected PreviousServicesError::MultipleServiceEvents"),
        }
    }

    #[test]
    fn previous_services_get_prev_service_idx_empty() {
        let target_miles = 100000;
        let target_date = NaiveDate::from_ymd_opt(2030, 1, 1).unwrap();
        assert!(
            !PREV_SERVICES
                .iter()
                .any(|service| service.miles() == target_miles || service.date() == target_date)
        );
        let found = get_prev_service_idx(&PREV_SERVICES, Some(target_miles), Some(target_date))
            .unwrap_err();

        match found {
            PreviousServicesError::ServiceEventNotFound => (),
            _ => panic!("Expected PreviousServicesError::ServiceEventNotFound"),
        }
    }

    #[test]
    fn previous_services_filter_prev_services_idxs_err() {
        assert!(filter_prev_services_idxs(&PREV_SERVICES, None, None).is_err());
    }

    #[test]
    fn previous_services_filter_miles() {
        let found = filter_prev_services_idxs(&PREV_SERVICES, Some(MILES.miles()), None).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0], 0);
    }

    #[test]
    fn previous_services_filter_date() {
        let found = filter_prev_services_idxs(&PREV_SERVICES, None, Some(DATE.date())).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0], 1);
    }

    #[test]
    fn previous_services_filter_miles_and_date() {
        let found = filter_prev_services_idxs(
            &PREV_SERVICES,
            Some(MILES_DATE.miles()),
            Some(MILES_DATE.date()),
        )
        .unwrap();
        assert_eq!(found.len(), 2);
        assert_eq!(found[0], 2);
        assert_eq!(found[1], 3);
    }
}
