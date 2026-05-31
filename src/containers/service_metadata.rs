use crate::containers::services::{ServiceEvent, ServiceInterval};
use crate::errors::ServiceMetadataError;
use chrono::NaiveDate;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ServiceMetdata {
    name: String,
    service_interval: ServiceInterval,
    next_service: ServiceEvent,
    previous_services: Option<Vec<ServiceEvent>>,
    notes: Option<Vec<String>>,
}

impl ServiceMetdata {
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
            previous_services,
            notes,
        }
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn set_name(&mut self, name: &str) {
        self.name = name.into()
    }

    pub(crate) fn service_interval(&self) -> ServiceInterval {
        self.service_interval
    }

    pub(crate) fn set_service_interval_miles(&mut self, miles: u32) {
        self.service_interval.miles = miles
    }

    pub(crate) fn set_service_interval_months(&mut self, months: u32) {
        self.service_interval.months = months
    }

    pub(crate) fn next_service(&self) -> ServiceEvent {
        self.next_service
    }

    pub(crate) fn set_next_service(&mut self, miles: u32, date: NaiveDate) {
        self.next_service.set_miles(miles);
        self.next_service.set_date(date);
    }

    pub(crate) fn exetend_notes(&mut self, extension: Vec<String>) {
        match &mut self.notes {
            Some(notes) => notes.extend(extension),
            None => self.notes = Some(extension),
        }
    }

    fn get_notes(&mut self) -> Result<&mut Vec<String>, ServiceMetadataError> {
        match &mut self.notes {
            Some(notes) => Ok(notes),
            None => Err(ServiceMetadataError::NotesNotSet),
        }
    }

    pub(crate) fn replace_note(
        &mut self,
        index: usize,
        contents: &str,
    ) -> Result<(), ServiceMetadataError> {
        let notes = self.get_notes()?;
        match notes.get_mut(index) {
            Some(val) => {
                *val = contents.into();
                Ok(())
            }
            None => Err(ServiceMetadataError::NoteIndexNotFound(index)),
        }
    }

    pub(crate) fn remove_note(&mut self, index: usize) -> Result<(), ServiceMetadataError> {
        let notes = self.get_notes()?;
        match index < notes.len() {
            true => {
                notes.remove(index);
                Ok(())
            }
            false => Err(ServiceMetadataError::NoteIndexOutOfRange(
                index,
                notes.len(),
            )),
        }
    }

    pub(crate) fn insert_note(
        &mut self,
        index: usize,
        contents: &str,
    ) -> Result<(), ServiceMetadataError> {
        let notes = self.get_notes()?;
        match index <= notes.len() {
            true => {
                notes.insert(index, contents.into());
                Ok(())
            }
            false => Err(ServiceMetadataError::NoteIndexOutOfRange(
                index,
                notes.len(),
            )),
        }
    }

    pub(crate) fn clear_notes(&mut self) {
        self.notes = None
    }

    pub(crate) fn add_service_event(&mut self, miles: u32, date: NaiveDate) {
        let service_event = ServiceEvent::new(miles, date);
        match &mut self.previous_services {
            None => self.previous_services = Some(vec![service_event]),
            Some(previous_services) => previous_services.push(service_event),
        }
    }

    fn get_previous_services(&mut self) -> Result<&mut Vec<ServiceEvent>, ServiceMetadataError> {
        match &mut self.previous_services {
            Some(prev_services) => Ok(prev_services),
            None => Err(ServiceMetadataError::PreviousServicesNotSet),
        }
    }

    pub(crate) fn replace_service_event(
        &mut self,
        curr_miles: Option<u32>,
        curr_date: Option<NaiveDate>,
        new_miles: Option<u32>,
        new_date: Option<NaiveDate>,
    ) -> Result<(), ServiceMetadataError> {
        info!("Attempting to replace previous service.");
        let prev_services = self.get_previous_services()?;
        let idx = get_prev_service_idx(prev_services, curr_miles, curr_date)?;

        // Note - .get_mut() should never error (return None) because get_prev_service_idx ensures we have the service event
        // Leaving error as fallback
        prev_services
            .get_mut(idx)
            .ok_or(ServiceMetadataError::ServiceEventNotFound)?
            .update(new_miles, new_date);

        info!("Replaced previous service.");

        Ok(())
    }

    pub(crate) fn remove_service_event(
        &mut self,
        miles: Option<u32>,
        date: Option<NaiveDate>,
    ) -> Result<(), ServiceMetadataError> {
        info!("Attempting to remove service event.");
        let prev_services = self.get_previous_services()?;
        let idx = get_prev_service_idx(prev_services, miles, date)?;

        // Remove can panic
        // get_prev_service_idx shuould ensure this index exists so the remove call is safe
        prev_services.remove(idx);

        info!("Removed service event.");

        Ok(())
    }

    pub(crate) fn clear_previous_services(&mut self) {
        info!("Clearing previous services");
        self.previous_services = None
    }
}

fn get_prev_service_idx(
    prev_services: &[ServiceEvent],
    miles: Option<u32>,
    date: Option<NaiveDate>,
) -> Result<usize, ServiceMetadataError> {
    let idxs = filter_prev_services_idxs(prev_services, miles, date)?;
    if idxs.is_empty() {
        debug!("No previous events found for provided filter.");
        Err(ServiceMetadataError::ServiceEventNotFound)
    } else if idxs.len() > 1 {
        debug!("Found multiple previous service events for provided filter.");
        Err(ServiceMetadataError::MultipleServiceEvents)
    } else {
        debug!("Found service event.");
        Ok(idxs[0])
    }
}

fn filter_prev_services_idxs(
    prev_services: &[ServiceEvent],
    miles: Option<u32>,
    date: Option<NaiveDate>,
) -> Result<Vec<usize>, ServiceMetadataError> {
    let result: Vec<usize> = match (miles, date) {
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
        (None, None) => {
            return Err(ServiceMetadataError::InvalidOptionalArgs);
        }
    };

    Ok(result)
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

        match &self.previous_services {
            Some(previous_services) => {
                write!(f, "\nPrevious Services:")?;
                previous_services.iter().try_for_each(|service| {
                    write!(f, "\n  - {}/{} miles", service.date(), service.miles())
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
