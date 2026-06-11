use super::notes::Notes;
use crate::containers::previous_services::PreviousServices;
use crate::containers::services::{ServiceEvent, ServiceInterval};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ServiceMetdata {
    name: String,
    service_interval: ServiceInterval,
    next_service: ServiceEvent,
    previous_services: PreviousServices,
    notes: Notes,
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
            previous_services: PreviousServices::new(previous_services),
            notes: Notes::new(notes),
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

    pub(crate) fn notes_mut(&mut self) -> &mut Notes {
        &mut self.notes
    }

    pub(crate) fn prev_services_mut(&mut self) -> &mut PreviousServices {
        &mut self.previous_services
    }

    pub(crate) fn set_next_service(&mut self, miles: u32, date: NaiveDate) {
        self.next_service.set_miles(miles);
        self.next_service.set_date(date);
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
