use crate::cmds::ServiceInterval as InitServiceInterval;
use crate::cmds::{NextService, PreviousService};
use crate::dates::months_to_days_floored;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub(crate) struct ServiceEvent {
    miles: u32,
    date: NaiveDate,
}

impl ServiceEvent {
    pub(crate) fn new(miles: u32, date: NaiveDate) -> Self {
        Self { miles, date }
    }

    pub(crate) fn miles(&self) -> u32 {
        self.miles
    }

    pub(super) fn set_miles(&mut self, miles: u32) {
        self.miles = miles
    }

    pub(crate) fn date(&self) -> NaiveDate {
        self.date
    }

    pub(super) fn set_date(&mut self, date: NaiveDate) {
        self.date = date
    }

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

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub(crate) struct ServiceInterval {
    pub(super) miles: u32,
    pub(super) months: u32,
}

impl ServiceInterval {
    pub(crate) fn new(miles: u32, months: u32) -> Self {
        Self { miles, months }
    }

    pub(crate) fn miles(&self) -> u32 {
        self.miles
    }

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
