use chrono::NaiveDate;
use clap::Args;

#[derive(Clone, Copy, Debug, Args)]
#[command(about = "Get next services for a specific mileage interval, date, or both")]
pub struct NextServices {
    #[arg(short, long, help = "Current mileage on vehicle")]
    current_mileage: u32,

    #[arg(
        short,
        long,
        help = "How many miles ahead form current mileage to look for maintenance items."
    )]
    miles: Option<u32>,

    #[arg(
        short,
        long,
        help = "Threshold date by which maintenance items should completed."
    )]
    date_threhold: Option<NaiveDate>,
}
