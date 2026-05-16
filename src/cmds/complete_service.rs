use chrono::{Local, NaiveDate};
use clap::Args;

#[derive(Clone, Debug, Args)]
#[command(about = "Complete a service on specific day and mileage")]
pub struct CompleteService {
    #[arg(short, long, help = "Service id to complete")]
    id: String,

    #[arg(short, long, help = "Mileage on vehicle upon service completion")]
    mileage: u32,

    #[arg(short, long, default_value_t = Local::now().date_naive(), help = "Date on which service was completed")]
    date: NaiveDate,
}
