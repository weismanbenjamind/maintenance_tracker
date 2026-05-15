use crate::errors::ServiceOptionsError;
use chrono::NaiveDate;
use clap::Args;

#[derive(Clone, Copy, Debug, Args)]
#[group(required = true)]
pub struct ServiceOptions {
    #[arg(
        short,
        long,
        help = "Maintenance due by a specific mileage threshold (inclusive)"
    )]
    miles_threshold: Option<u32>,

    #[arg(
        short,
        long,
        help = "Maintenance due by a specific date threshold (inclusive)"
    )]
    date_threhold: Option<NaiveDate>,

    #[arg(
        short,
        long,
        help = "Current mileage on vehicle",
        requires = "miles_from_current"
    )]
    current_mileage: Option<u32>,

    #[arg(
        short = 'l',
        long,
        help = "Maintenance that needs done by a certain number of miles from the current vehicle mileage",
        requires = "current_mileage"
    )]
    miles_from_current: Option<u32>,

    #[arg(
        short = 'i',
        long,
        help = "Maintenance that needs done be a certain number of months from today"
    )]
    months_from_today: Option<u32>,

    #[arg(
        short,
        long,
        help = "Override 'today' when using the --months-from-today arg",
        requires = "months_from_today"
    )]
    today: Option<NaiveDate>,
}

#[allow(dead_code)]
impl ServiceOptions {
    pub fn new(
        miles_threshold: Option<u32>,
        date_threhold: Option<NaiveDate>,
        current_mileage: Option<u32>,
        miles_from_current: Option<u32>,
        months_from_today: Option<u32>,
        today: Option<NaiveDate>,
    ) -> Result<Self, ServiceOptionsError> {
        if miles_from_current.is_some() && current_mileage.is_none() {
            return Err(ServiceOptionsError::InvalidMileageArgs);
        }

        if miles_from_current.is_none() && current_mileage.is_some() {
            return Err(ServiceOptionsError::InvalidMileageArgs);
        }

        if today.is_some() && months_from_today.is_none() {
            return Err(ServiceOptionsError::InvalidDateArgs);
        }

        Ok(Self {
            miles_threshold,
            date_threhold,
            current_mileage,
            miles_from_current,
            months_from_today,
            today,
        })
    }

    pub fn miles_threshold(&self) -> Option<u32> {
        self.miles_threshold
    }

    pub fn date_threhold(&self) -> Option<NaiveDate> {
        self.date_threhold
    }

    pub fn current_mileage(&self) -> Option<u32> {
        self.current_mileage
    }

    pub fn miles_from_current(&self) -> Option<u32> {
        self.miles_from_current
    }

    pub fn months_from_today(&self) -> Option<u32> {
        self.months_from_today
    }

    pub fn today(&self) -> Option<NaiveDate> {
        self.today
    }
}
