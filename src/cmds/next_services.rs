use super::service_options::ServiceOptions;
use clap::Args;

#[derive(Clone, Copy, Debug, Args)]
#[command(
    about = "Get next services for a specific mileage interval/threshold and/or date interval/threshold"
)]
pub struct NextServices {
    #[command(flatten)]
    service_options: ServiceOptions,
}

impl NextServices {
    pub fn new(service_options: ServiceOptions) -> Self {
        Self { service_options }
    }

    pub fn service_options(self) -> ServiceOptions {
        self.service_options
    }
}
