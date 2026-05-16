use clap::Args;

#[derive(Clone, Debug, Args)]
#[command(about = "Show details about a specific service")]
pub struct Detail {
    #[arg(short, long, help = "Service to get details about")]
    name: String,
}

impl Detail {
    pub fn new(name: &str) -> Self {
        Self { name: name.into() }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
