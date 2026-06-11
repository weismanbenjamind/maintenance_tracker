use super::services::ServiceEvent;
use crate::errors::PreviousServicesError;
use chrono::NaiveDate;
use log::{debug, info};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub(crate) struct PreviousServices {
    service_events: Option<Vec<ServiceEvent>>,
}

impl PreviousServices {
    pub(crate) fn new(service_events: Option<Vec<ServiceEvent>>) -> Self {
        Self { service_events }
    }

    pub(crate) fn add(&mut self, miles: u32, date: NaiveDate) {
        let service_event = ServiceEvent::new(miles, date);
        match &mut self.service_events {
            Some(service_events) => service_events.push(service_event),
            None => self.service_events = Some(vec![service_event]),
        }
    }

    pub(crate) fn service_events(&self) -> Option<&[ServiceEvent]> {
        self.service_events.as_deref()
    }

    fn get_previous_services_mut(
        &mut self,
    ) -> Result<&mut Vec<ServiceEvent>, PreviousServicesError> {
        match &mut self.service_events {
            Some(service_events) => Ok(service_events),
            None => Err(PreviousServicesError::PreviousServicesNotSet),
        }
    }

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

    pub(crate) fn remove(
        &mut self,
        miles: Option<u32>,
        date: Option<NaiveDate>,
    ) -> Result<(), PreviousServicesError> {
        info!("Attempting to remove service event");

        let prev_services = self.get_previous_services_mut()?;
        let idx = get_prev_service_idx(prev_services, miles, date)?;

        // swap_remove can panic
        // get_prev_service_idx should ensure this index exists so the removal call is safe
        prev_services.remove(idx);

        info!("Removed service event");

        Ok(())
    }

    pub(crate) fn clear(&mut self) {
        self.service_events = None
    }
}

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
