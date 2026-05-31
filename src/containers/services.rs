use crate::cmds::init::ServiceInterval as InitServiceInterval;
use crate::cmds::init::{NextService, PreviousService};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub struct ServiceEvent {
    pub(super) miles: u32,
    pub(super) date: NaiveDate,
}

impl ServiceEvent {
    pub(super) fn update(&mut self, miles: Option<u32>, date: Option<NaiveDate>) {
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

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct ServiceInterval {
    pub(super) miles: u32,
    pub(super) months: u32,
}

impl ServiceInterval {
    pub fn miles(&self) -> u32 {
        self.miles
    }

    pub fn months(&self) -> u32 {
        self.months
    }

    pub fn days(&self) -> i64 {
        // Floor to prevent accidentally going overdue on maintenance
        (self.months as f64 / 12.0 * 365.0).floor() as i64
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
