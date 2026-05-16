use crate::errors::DiffOptionsError;
use chrono::NaiveDate;
use clap::Args;

#[derive(Clone, Debug, Args)]
#[command(
    about = "Get mileage, date, mileage difference, date difference, or any combination for a given service or all services"
)]
pub struct Diff {
    #[arg(
        short,
        long,
        help = "ID of service. If omitted, all differences will be calculated for all services."
    )]
    id: Option<String>,

    #[command(flatten)]
    diff_options: DiffOptions,
}

impl Diff {
    pub fn new(id: Option<&str>, diff_options: DiffOptions) -> Self {
        Self {
            id: id.map(|id| id.into()),
            diff_options,
        }
    }

    pub fn diff_options(self) -> DiffOptions {
        self.diff_options
    }
}

#[derive(Clone, Copy, Debug, Args)]
#[group(required = true)]
pub struct DiffOptions {
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
        short = 'f',
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
impl DiffOptions {
    pub fn new(
        miles_threshold: Option<u32>,
        date_threhold: Option<NaiveDate>,
        current_mileage: Option<u32>,
        miles_from_current: Option<u32>,
        months_from_today: Option<u32>,
        today: Option<NaiveDate>,
    ) -> Result<Self, DiffOptionsError> {
        if miles_from_current.is_some() && current_mileage.is_none() {
            return Err(DiffOptionsError::InvalidMileageArgs);
        }

        if miles_from_current.is_none() && current_mileage.is_some() {
            return Err(DiffOptionsError::InvalidMileageArgs);
        }

        if today.is_some() && months_from_today.is_none() {
            return Err(DiffOptionsError::InvalidDateArgs);
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
