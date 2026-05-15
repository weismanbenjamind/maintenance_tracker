use super::service_options::ServiceOptions;
use clap::Args;

#[derive(Clone, Debug, Args)]
#[command(
    about = "Get mileage, date, mileage difference, date difference, or any combination for a given service"
)]
pub struct NextService {
    #[arg(short, long, help = "Service to get intervals for")]
    name: String,

    #[command(flatten)]
    service_options: ServiceOptions,
}

impl NextService {
    pub fn new(name: &str, service_options: ServiceOptions) -> Self {
        Self {
            name: name.into(),
            service_options,
        }
    }

    pub fn service_options(self) -> ServiceOptions {
        self.service_options
    }
}
