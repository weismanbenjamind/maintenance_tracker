// Using a unit (static) struct hre to follow all command patters
use clap::Args;

#[derive(Debug, Clone, Args)]
#[command(about = "List all services and their ids")]
pub struct List;
