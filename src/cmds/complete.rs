use chrono::{Local, NaiveDate};
use clap::Args;

#[derive(Clone, Debug, Args)]
#[command(about = "Complete a service on specific day and mileage")]
pub struct Complete {
    #[arg(short, long, help = "ID of service")]
    id: String,

    #[arg(short, long, help = "Mileage on vehicle upon service completion")]
    mileage: u32,

    #[arg(short, long, default_value_t = Local::now().date_naive(), help = "Date on which service was completed")]
    date: NaiveDate,
}

impl Complete {
    pub fn new(id: &str, mileage: u32, date: NaiveDate) -> Self {
        Self {
            id: id.into(),
            mileage,
            date,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn mileage(&self) -> u32 {
        self.mileage
    }

    pub fn date(&self) -> NaiveDate {
        self.date
    }

    pub fn run(self) {}
}
