use crate::containers::{ServiceEvent, ServiceInterval};
use crate::errors::ContainersError;
use chrono::NaiveDate;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServiceMetdata {
    name: String,
    service_interval: ServiceInterval,
    next_service: ServiceEvent,
    previous_services: Option<Vec<ServiceEvent>>,
    notes: Option<Vec<String>>,
}

impl ServiceMetdata {
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.into()
    }

    pub fn service_interval(&self) -> ServiceInterval {
        self.service_interval
    }

    pub fn set_service_interval_miles(&mut self, miles: u32) {
        self.service_interval.miles = miles
    }

    pub fn set_service_interval_months(&mut self, months: u32) {
        self.service_interval.months = months
    }

    pub fn next_service(&self) -> ServiceEvent {
        self.next_service
    }

    pub fn set_next_service(&mut self, miles: u32, date: NaiveDate) {
        self.next_service.miles = miles;
        self.next_service.date = date;
    }

    // TODO - See if can de-dupe any notes methods
    pub fn exetend_notes(&mut self, extension: Vec<String>) {
        match &mut self.notes {
            Some(notes) => notes.extend(extension),
            None => self.notes = Some(extension),
        }
    }

    pub fn replace_note(&mut self, index: usize, contents: &str) {
        match &mut self.notes {
            Some(notes) => match notes.get_mut(index) {
                Some(val) => *val = contents.into(),
                None => notes.push(contents.into()),
            },
            None => self.notes = Some(vec![contents.into()]),
        }
    }

    pub fn remove_note(&mut self, index: usize) {
        if let Some(notes) = &mut self.notes
            && index < notes.len()
        {
            notes.remove(index);
        }
    }

    pub fn insert_note(&mut self, index: usize, contents: &str) {
        match &mut self.notes {
            Some(notes) => match index > notes.len() {
                true => notes.push(contents.into()),
                false => notes.insert(index, contents.into()),
            },
            None => self.notes = Some(vec![contents.into()]),
        }
    }

    pub fn clear_notes(&mut self) {
        self.notes = None
    }

    pub fn add_service_event(&mut self, miles: u32, date: NaiveDate) {
        let service_event = ServiceEvent { miles, date };
        match &mut self.previous_services {
            None => self.previous_services = Some(vec![service_event]),
            Some(previous_services) => previous_services.push(service_event),
        }
    }

    pub fn replace_service_event(
        &mut self,
        curr_miles: Option<u32>,
        curr_date: Option<NaiveDate>,
        new_miles: Option<u32>,
        new_date: Option<NaiveDate>,
    ) -> Result<(), ContainersError> {
        info!("Attempting to replace previous service.");
        match &mut self.previous_services {
            None => {
                info!("No previous services. Skipping replace operation");
                Err(ContainersError::ServiceEventNotFound)
            }
            Some(prev_services) => {
                let idx = get_prev_service_idx(prev_services, curr_miles, curr_date)?;

                // Note - .get_mut() should never error (return None) because get_prev_service_idx ensures we have the service event
                // Leaving error as fallback
                prev_services
                    .get_mut(idx)
                    .ok_or(ContainersError::ServiceEventNotFound)?
                    .update(new_miles, new_date);

                info!("Replaced previous service.");

                Ok(())
            }
        }
    }

    pub fn remove_service_event(
        &mut self,
        miles: Option<u32>,
        date: Option<NaiveDate>,
    ) -> Result<(), ContainersError> {
        info!("Attempting to remove service event.");
        match &mut self.previous_services {
            None => {
                info!("No previous services. Skipping remove operation.");
                Err(ContainersError::ServiceEventNotFound)
            }
            Some(prev_services) => {
                let idx = get_prev_service_idx(prev_services, miles, date)?;

                // Remove can panic
                // get_prev_service_idx shuould ensure this index exists so the remove call is safe
                prev_services.remove(idx);

                info!("Removed service event.");

                Ok(())
            }
        }
    }

    pub fn clear_previous_services(&mut self) {
        info!("Clearing previous services");
        self.previous_services = None
    }
}

fn get_prev_service_idx(
    prev_services: &[ServiceEvent],
    miles: Option<u32>,
    date: Option<NaiveDate>,
) -> Result<usize, ContainersError> {
    let idxs = filter_prev_services_idxs(prev_services, miles, date)?;
    if idxs.is_empty() {
        debug!("No previous events found for provided filter.");
        Err(ContainersError::ServiceEventNotFound)
    } else if idxs.len() > 1 {
        debug!("Found multiple previous service events for provided filter.");
        Err(ContainersError::MultipleServiceEvents)
    } else {
        debug!("Found service event.");
        Ok(idxs[0])
    }
}

fn filter_prev_services_idxs(
    prev_services: &[ServiceEvent],
    miles: Option<u32>,
    date: Option<NaiveDate>,
) -> Result<Vec<usize>, ContainersError> {
    let result: Vec<usize> = match (miles, date) {
        (Some(miles), None) => prev_services
            .iter()
            .enumerate()
            .filter(|(_idx, service_event)| service_event.miles == miles)
            .map(|(idx, _service_event)| idx)
            .collect(),
        (None, Some(date)) => prev_services
            .iter()
            .enumerate()
            .filter(|(_idx, service_event)| service_event.date == date)
            .map(|(idx, _service_event)| idx)
            .collect(),
        (Some(miles), Some(date)) => prev_services
            .iter()
            .enumerate()
            .filter(|(_idx, service_event)| {
                service_event.miles == miles && service_event.date == date
            })
            .map(|(idx, _service_event)| idx)
            .collect(),
        (None, None) => {
            return Err(ContainersError::InvalidOptionalArgs);
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
