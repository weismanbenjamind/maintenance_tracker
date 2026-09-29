//! # Previous services
//!
//! Houses args, comands and utilities for updating a previos service

use chrono::{Local, NaiveDate};
use clap::{Args, Subcommand};

/// Args for appending a previous service
#[derive(Clone, Copy, Debug, Args)]
#[command(about = "Append a service")]
pub(super) struct Append {
    #[arg(help = "Mileage of service")]
    miles: u32,

    #[arg(short, long, default_value_t = Local::now().date_naive(), help = "Date of service")]
    date: NaiveDate,
}

impl Append {
    /// Build new append args.
    /// Only used for testing.
    #[cfg(test)]
    pub(super) fn new(miles: u32, date: NaiveDate) -> Self {
        Self { miles, date }
    }

    /// Get a copy of the miles arg
    pub(super) fn miles(&self) -> u32 {
        self.miles
    }

    /// Get a copy of the date arg
    pub(super) fn date(&self) -> NaiveDate {
        self.date
    }
}

/// Args for replacing a service
#[derive(Clone, Copy, Debug, Args)]
#[command(about = "Replace a service")]
pub(super) struct Replace {
    #[command(flatten)]
    curr_service_specs: CurrServiceSpecs,

    #[command(flatten)]
    updated_service_specs: UpdatedServiceSpecs,
}

impl Replace {
    /// Build a Replace args struct.
    /// Only used for testing.
    /// Will panic if both curr_service_miles and curr_service_date are None
    /// or if update_service_miles and updated_service_date are None.
    /// If updating to use this function outside of tests should refactor this function
    /// to return a result.
    #[cfg(test)]
    pub(super) fn new(
        curr_service_miles: Option<u32>,
        curr_service_date: Option<NaiveDate>,
        update_service_miles: Option<u32>,
        updated_service_date: Option<NaiveDate>,
    ) -> Self {
        // IF REFACTORING TO USE THIS OUTSIDE OF ONLY TESTS REMOVE THIS PANIC
        if curr_service_miles.is_none() && curr_service_date.is_none() {
            panic!("One of curr_service_miles or curr_service_date must be Some");
        }

        // IF REFACTORING TO USE THIS OUTSIDE OF ONLY TESTS REMOVE THIS PANIC
        if update_service_miles.is_none() && updated_service_date.is_none() {
            panic!("One of update_service_miles or updated_service_date must be Some");
        }

        Self {
            curr_service_specs: CurrServiceSpecs {
                miles: curr_service_miles,
                date: curr_service_date,
            },
            updated_service_specs: UpdatedServiceSpecs {
                new_miles: update_service_miles,
                new_date: updated_service_date,
            },
        }
    }

    /// Consume the replace args and convert them into a tuple in the format
    /// (curr_service_specs.miles, curr_service_specs.date, updated_service_specs.new_miles updated_service_specs.new_date)
    pub(super) fn into_parts(
        self,
    ) -> (
        Option<u32>,
        Option<NaiveDate>,
        Option<u32>,
        Option<NaiveDate>,
    ) {
        (
            self.curr_service_specs.miles,
            self.curr_service_specs.date,
            self.updated_service_specs.new_miles,
            self.updated_service_specs.new_date,
        )
    }
}

/// Args for removing a service
#[derive(Clone, Copy, Debug, Args)]
#[command(about = "Remove a service")]
pub(super) struct Remove {
    #[command(flatten)]
    service_specs: CurrServiceSpecs,
}

impl Remove {
    /// Build a Remoave args struct.
    /// Only used for testing.
    /// Will panic if both miles and date are None
    /// If updating to use this function outside of tests should refactor this function
    /// to return a result.
    #[cfg(test)]
    pub(super) fn new(miles: Option<u32>, date: Option<NaiveDate>) -> Self {
        // IF REFACTORING TO USE THIS OUTSIDE OF ONLY TESTS REMOVE THIS PANIC
        if miles.is_none() && date.is_none() {
            panic!("One of miles or date must be Some.")
        }

        Self {
            service_specs: CurrServiceSpecs { miles, date },
        }
    }

    /// Consume the remove args and convert them into a tuple in the format
    /// (service_specs.miles, service_specs.date)
    pub(super) fn into_parts(self) -> (Option<u32>, Option<NaiveDate>) {
        (self.service_specs.miles, self.service_specs.date)
    }
}

/// Unit struct for clearing args.
/// Only here to match pattern with args structs.
#[derive(Clone, Copy, Debug, Args)]
#[command(about = "Clear all services")]
pub(super) struct Clear;

/// Struct to hold current service specs
#[derive(Clone, Copy, Debug, Args)]
#[group(required = true, multiple = true)]
struct CurrServiceSpecs {
    #[arg(short, long, help = "Mileage on vehicle when service was performed")]
    miles: Option<u32>,

    #[arg(short, long, help = "Date when service was performed")]
    date: Option<NaiveDate>,
}

/// Struct to hold updated service specs
#[derive(Clone, Copy, Debug, Args)]
#[group(required = true, multiple = true)]
struct UpdatedServiceSpecs {
    #[arg(short, long, help = "Mileage on vehicle to update service to")]
    new_miles: Option<u32>,

    #[arg(short = 'w', long, help = "Date to update service to")]
    new_date: Option<NaiveDate>,
}

/// Enum to house all commands and their args
/// for upating a previous service
#[derive(Clone, Copy, Debug, Subcommand)]
pub(crate) enum UpdatePreviousServicesCmd {
    Append(Append),
    Replace(Replace),
    Remove(Remove),
    Clear(Clear),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prev_services_new() {
        let miles = 75000;
        let date = NaiveDate::from_ymd_opt(2026, 6, 7).unwrap();
        let new = Append::new(miles, date);
        assert_eq!(new.miles, miles);
        assert_eq!(new.date, date);
    }

    #[test]
    fn prev_services_append_getters() {
        let miles = 75000;
        let date = NaiveDate::from_ymd_opt(2026, 6, 7).unwrap();
        let cmd = Append { miles, date };
        assert_eq!(cmd.miles(), miles);
        assert_eq!(cmd.date(), date);
    }

    #[test]
    fn pre_service_replace_new() {
        let miles = Some(75000);
        let new = Replace::new(miles, None, miles, None);
        assert_eq!(new.curr_service_specs.miles, miles);
        assert!(new.curr_service_specs.date.is_none());
        assert_eq!(new.updated_service_specs.new_miles, miles);
        assert!(new.updated_service_specs.new_date.is_none());
    }

    #[test]
    fn prev_services_replace_into_part() {
        let miles = Some(75000);
        let date = None;
        let curr_service_specs = CurrServiceSpecs { miles, date };
        let updated_service_specs = UpdatedServiceSpecs {
            new_miles: miles,
            new_date: date,
        };
        let args = Replace {
            curr_service_specs,
            updated_service_specs,
        };
        assert_eq!(args.into_parts(), (miles, date, miles, date))
    }

    #[test]
    fn prev_services_remove_new() {
        let miles = Some(75000);
        let args = Remove::new(miles, None);
        assert_eq!(args.service_specs.miles, miles);
        assert!(args.service_specs.date.is_none());
    }

    #[test]
    fn prev_services_remove_into_parts() {
        let miles = Some(75000);
        let date = None;
        let args = Remove {
            service_specs: CurrServiceSpecs { miles, date },
        };
        assert_eq!(args.into_parts(), (miles, date));
    }
}
