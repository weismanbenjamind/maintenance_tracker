//! # Previous Services
//!
//! Houses functionality for dealing with previous services of a maintenance item

use chrono::NaiveDate;
use log::{debug, info};
use serde::{Deserialize, Serialize};

use super::services::ServiceEvent;
use crate::errors::PreviousServicesError;

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

        if prev_services.is_empty() {
            self.clear();
        }

        info!("Removed service event");

        Ok(())
    }

    pub(crate) fn clear(&mut self) {
        self.service_events = None
    }

    /// Get the most recent service if it exists. If no previous service has been
    /// performed returns `None`.
    pub(crate) fn get_most_recent(&self) -> Option<ServiceEvent> {
        self.service_events
            .as_deref()?
            .iter()
            .max_by_key(|s| s.date())
            .copied()
    }

    /// Gets the number of previous services if set.
    /// If no previous services returns None.
    /// Only used for testing.
    #[cfg(test)]
    pub(crate) fn len(&self) -> Option<usize> {
        self.service_events
            .as_deref()
            .and_then(|service_events| Some(service_events.len()))
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
    use chrono::NaiveDate;

    use super::*;

    const MILES: ServiceEvent =
        ServiceEvent::new(70000, NaiveDate::from_ymd_opt(2026, 6, 1).unwrap());
    const DATE: ServiceEvent =
        ServiceEvent::new(74000, NaiveDate::from_ymd_opt(2026, 10, 14).unwrap());
    const MILES_DATE: ServiceEvent =
        ServiceEvent::new(77000, NaiveDate::from_ymd_opt(2027, 2, 14).unwrap());
    const OVERLAP: ServiceEvent = ServiceEvent::new(MILES_DATE.miles(), MILES_DATE.date());
    const PREV_SERVICES: [ServiceEvent; 4] = [MILES, DATE, MILES_DATE, OVERLAP];

    /// Builds PreviousServices with some service events present
    fn build_previous_services_some() -> PreviousServices {
        PreviousServices {
            service_events: Some(PREV_SERVICES.into()),
        }
    }

    /// Builds PreviousServices with no service events present
    fn build_previous_services_none() -> PreviousServices {
        PreviousServices {
            service_events: None,
        }
    }

    #[test]
    fn previous_services_previous_services_remove_prev_services_no_idx() {
        let mut previous_services = build_previous_services_some();
        let target_miles = 1;
        assert!(
            previous_services
                .service_events
                .as_deref()
                .unwrap()
                .iter()
                .all(|service_event| service_event.miles() != target_miles)
        );

        let found = previous_services
            .remove(Some(target_miles), None)
            .unwrap_err();

        match found {
            PreviousServicesError::ServiceEventNotFound => (),
            _ => panic!("Expected PreviousServicesError::ServiceEventNotFound"),
        }
    }

    #[test]
    fn previous_services_previous_services_remove_prev_services_not_set() {
        let mut prev_services = build_previous_services_none();
        let found = prev_services
            .remove(Some(1000), Some(NaiveDate::default()))
            .unwrap_err();

        match found {
            PreviousServicesError::PreviousServicesNotSet => (),
            _ => panic!("Expected PreviousServicesError::PreviousServicesNotSet"),
        }
    }

    #[test]
    fn previous_services_previous_services_remove_ok() {
        let mut previous_services = build_previous_services_some();
        let mut original_length: Option<usize> = None;
        previous_services
            .service_events
            .as_deref()
            .inspect(|service_events| original_length = Some(service_events.len()));
        let original_length = original_length.unwrap();

        previous_services
            .remove(Some(MILES.miles()), Some(MILES.date()))
            .unwrap();

        let service_events = previous_services.service_events.unwrap();
        assert!(original_length > service_events.len());
        assert!(
            service_events
                .iter()
                .all(|service_event| *service_event != MILES)
        );
    }

    #[test]
    fn previous_services_previous_services_remove_all() {
        let miles = 70000;
        let prev_service = ServiceEvent::new(miles, NaiveDate::from_ymd_opt(2026, 6, 1).unwrap());
        let mut previous_services = PreviousServices::new(Some(vec![prev_service]));

        previous_services.remove(Some(MILES.miles()), None).unwrap();
        assert!(previous_services.service_events.is_none());
    }

    #[test]
    fn previous_services_previous_services_replace_previous_services_no_idx() {
        let mut prev_services = build_previous_services_some();
        let target_miles = 1;

        assert!(
            prev_services
                .service_events
                .as_deref()
                .unwrap()
                .iter()
                .all(|service_event| service_event.miles() != target_miles)
        );

        let found = prev_services
            .replace(Some(target_miles), None, Some(80000), None)
            .unwrap_err();

        match found {
            PreviousServicesError::ServiceEventNotFound => (),
            _ => panic!("Expected PreviousServicesError::ServiceEventNotFound error"),
        }
    }

    #[test]
    fn previous_services_previous_services_replace_previous_services_none() {
        let mut prev_services = build_previous_services_none();
        let found = prev_services
            .replace(Some(750000), None, Some(80000), None)
            .unwrap_err();

        match found {
            PreviousServicesError::PreviousServicesNotSet => (),
            _ => panic!("Expected PreviousServicesError::PreviousServicesNotSet error"),
        }
    }

    #[test]
    fn previous_services_previous_services_replace_miles_and_date() {
        let mut prev_services = build_previous_services_some();
        let miles_update = 80000;
        let date_update = NaiveDate::from_ymd_opt(2027, 6, 18).unwrap();
        let miles_to_update = MILES.miles();
        let date_to_update = MILES.date();
        assert_ne!(miles_update, miles_to_update);
        assert_ne!(date_update, date_to_update);

        prev_services
            .replace(
                Some(miles_to_update),
                Some(date_to_update),
                Some(miles_update),
                Some(date_update),
            )
            .unwrap();

        let found = prev_services.service_events.unwrap()[0];

        assert_eq!(found.miles(), miles_update);
        assert_eq!(found.date(), date_update);
    }

    #[test]
    fn previous_services_previous_services_replace_date() {
        let mut prev_services = build_previous_services_some();
        let date_update = NaiveDate::from_ymd_opt(2027, 6, 18).unwrap();
        let to_update = DATE.date();
        assert_ne!(date_update, to_update);

        prev_services
            .replace(None, Some(to_update), None, Some(date_update))
            .unwrap();

        assert_eq!(prev_services.service_events.unwrap()[1].date(), date_update);
    }

    #[test]
    fn previous_services_previous_services_replace_miles() {
        let mut prev_services = build_previous_services_some();
        let miles_update = 80000;
        let to_update = MILES.miles();
        assert_ne!(miles_update, to_update);

        prev_services
            .replace(Some(to_update), None, Some(miles_update), None)
            .unwrap();

        assert_eq!(
            prev_services.service_events.unwrap()[0].miles(),
            miles_update
        );
    }

    #[test]
    fn previous_services_previous_services_get_previous_services_mut_err() {
        let mut prev_services = build_previous_services_none();
        match prev_services.get_previous_services_mut().unwrap_err() {
            PreviousServicesError::PreviousServicesNotSet => (),
            _ => panic!("Expected PreviousServicesError::PreviousServicesNotSet"),
        }
    }

    #[test]
    fn previous_services_previous_services_get_previous_services_mut_ok() {
        let mut prev_services = build_previous_services_some();
        let found = prev_services.get_previous_services_mut().unwrap();
        let expected: Vec<ServiceEvent> = PREV_SERVICES.into();
        assert_eq!(*found, expected);
    }

    #[test]
    fn previous_services_previous_services_service_events_services_not_present() {
        let previous_services = build_previous_services_none();
        assert_eq!(previous_services.service_events(), None)
    }

    #[test]
    fn previous_services_previous_services_service_events_services_present() {
        let previous_services = build_previous_services_some();
        assert_eq!(previous_services.service_events().unwrap(), &PREV_SERVICES)
    }

    #[test]
    fn previous_services_previous_services_add_no_services() {
        let mut prev_services = build_previous_services_none();
        let addition = ServiceEvent::new(80000, NaiveDate::from_ymd_opt(2027, 2, 14).unwrap());
        prev_services.add(addition.miles(), addition.date());

        let updated_services = prev_services.service_events.unwrap();
        let new_len = updated_services.len();

        assert_eq!(new_len, 1);
        assert_eq!(updated_services[0], addition);
    }

    #[test]
    fn previous_services_previous_services_add_services_present() {
        let mut prev_services = build_previous_services_some();
        let original_len = prev_services.len().unwrap();
        let addition = ServiceEvent::new(80000, NaiveDate::from_ymd_opt(2027, 2, 14).unwrap());
        prev_services.add(addition.miles(), addition.date());

        let updated_services = prev_services.service_events.unwrap();
        let new_len = updated_services.len();

        assert_eq!(new_len, original_len + 1);
        assert_eq!(updated_services[new_len - 1], addition);
    }

    #[test]
    fn previous_services_previous_services_clear() {
        let mut prev_services = build_previous_services_some();
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
