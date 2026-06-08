use chrono::Local;
use chrono::NaiveDate;
use clap::{Args, Subcommand};

#[derive(Clone, Copy, Debug, Args)]
#[command(about = "Append a service")]
pub(super) struct Append {
    #[arg(short, long, help = "Mileage of service")]
    miles: u32,

    #[arg(short, long, default_value_t = Local::now().date_naive(), help = "Date of service")]
    date: NaiveDate,
}

impl Append {
    pub(super) fn miles(&self) -> u32 {
        self.miles
    }

    pub(super) fn date(&self) -> NaiveDate {
        self.date
    }
}

#[derive(Clone, Copy, Debug, Args)]
#[command(about = "Replace a service")]
pub(super) struct Replace {
    #[command(flatten)]
    curr_service_specs: CurrServiceSpecs,

    #[command(flatten)]
    updated_service_specs: UpdatedServiceSpecs,
}

impl Replace {
    pub(super) fn curr_miles(&self) -> Option<u32> {
        self.curr_service_specs.miles
    }

    pub(super) fn curr_date(&self) -> Option<NaiveDate> {
        self.curr_service_specs.date
    }

    pub(super) fn new_miles(&self) -> Option<u32> {
        self.updated_service_specs.new_miles
    }

    pub(super) fn new_date(&self) -> Option<NaiveDate> {
        self.updated_service_specs.new_date
    }
}

#[derive(Clone, Copy, Debug, Args)]
#[command(about = "Remove a service")]
pub(super) struct Remove {
    #[command(flatten)]
    service_specs: CurrServiceSpecs,
}

impl Remove {
    pub(super) fn miles(&self) -> Option<u32> {
        self.service_specs.miles
    }

    pub(super) fn date(&self) -> Option<NaiveDate> {
        self.service_specs.date
    }
}

#[derive(Clone, Copy, Debug, Args)]
#[command(about = "Clear all services")]
pub(super) struct Clear;

#[derive(Clone, Copy, Debug, Args)]
#[group(required = true, multiple = true)]
struct CurrServiceSpecs {
    #[arg(short, long, help = "Mileage on vehicle when service was performed")]
    miles: Option<u32>,

    #[arg(short, long, help = "Date when service was performed")]
    date: Option<NaiveDate>,
}

#[derive(Clone, Copy, Debug, Args)]
#[group(required = true, multiple = true)]
struct UpdatedServiceSpecs {
    #[arg(short, long, help = "Mileage on vehicle to update service to")]
    new_miles: Option<u32>,

    #[arg(short = 'w', long, help = "Date to update service to")]
    new_date: Option<NaiveDate>,
}

#[derive(Clone, Copy, Debug, Subcommand)]
pub(crate) enum UpdateServiceCmd {
    Append(Append),
    Replace(Replace),
    Remove(Remove),
    Clear(Clear),
}
