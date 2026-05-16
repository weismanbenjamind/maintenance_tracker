use clap::Args;

#[derive(Clone, Debug, Args)]
#[command(about = "Get the next service event for a given maintenance item")]
pub struct Next {
    #[arg(short, long, help = "ID of service")]
    id: String,
}

impl Next {
    pub fn new(id: &str) -> Self {
        Self { id: id.into() }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}
