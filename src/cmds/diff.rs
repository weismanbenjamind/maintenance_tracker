use chrono::NaiveDate;
use clap::Args;

#[derive(Clone, Debug, Args)]
#[command(
    about = "Get mileage, date, mileage difference, date difference, or any combination for a given service or all services"
)]
pub struct Diff {
    #[arg(
        help = "ID of service. If omitted, all differences will be calculated for all services."
    )]
    id: Option<String>,

    #[command(flatten)]
    diff_options: DiffOptions,
}

impl Diff {
    pub fn run(self) {}
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
