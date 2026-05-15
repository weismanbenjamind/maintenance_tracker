use clap::Args;

#[derive(Clone, Debug, Args)]
#[command(about = "Get details about a specific service")]
pub struct ServiceDetails {
    #[arg(short, long, help = "Service to get details about")]
    name: String,
}

impl ServiceDetails {
    pub fn new(name: &str) -> Self {
        Self { name: name.into() }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
