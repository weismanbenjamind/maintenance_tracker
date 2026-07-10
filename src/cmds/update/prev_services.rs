//! # Previous services
//!
//! Houses args, comands and utilities for updating a previos service

use chrono::Local;
use chrono::NaiveDate;
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
    fn prev_services_append_getters() {
        let miles = 75000;
        let date = NaiveDate::from_ymd_opt(2026, 6, 7).unwrap();
        let cmd = Append { miles, date };
        assert_eq!(cmd.miles(), miles);
        assert_eq!(cmd.date(), date);
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
    fn prev_services_remove_into_parts() {
        let miles = Some(75000);
        let date = None;
        let args = Remove {
            service_specs: CurrServiceSpecs { miles, date },
        };
        assert_eq!(args.into_parts(), (miles, date));
    }
}
