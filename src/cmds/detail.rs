use clap::Args;

#[derive(Clone, Debug, Args)]
#[command(about = "Show details about a specific service")]
pub struct Detail {
    #[arg(short, long, help = "ID of service")]
    id: String,
}

impl Detail {
    pub fn new(id: &str) -> Self {
        Self { id: id.into() }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}
