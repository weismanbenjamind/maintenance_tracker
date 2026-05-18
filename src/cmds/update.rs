use clap::{Args, Subcommand};

#[derive(Clone, Debug, Args)]
#[command(about = "Update a service")]
pub struct Update {
    #[arg(short, long, help = "ID of service to update")]
    id: String,

    #[arg(long, help = "Name of service")]
    name: Option<String>,

    #[arg(long, help = "Miles interval the service should be completed at")]
    miles_interval: Option<u32>,

    #[arg(long, help = "Monthly interval service should be completed at")]
    monthly_interval: Option<u32>,

    #[command(subcommand)]
    command: Option<UpdateVec>,
}

#[derive(Clone, Debug, Subcommand)]
pub enum UpdateVec {
    Notes(UpdateNotes),
}

#[derive(Clone, Debug, Args)]
#[command(about = "Update notes")]
pub struct UpdateNotes {
    #[command(subcommand)]
    cmd: UpdateNotesCmd,
}

#[derive(Clone, Debug, Subcommand)]
pub enum UpdateNotesCmd {
    Append(update_notes_cmds::Append),
    Replace(update_notes_cmds::Replace),
    Insert(update_notes_cmds::Insert),
    Remove(update_notes_cmds::Remove),
    Clear(update_notes_cmds::Clear),
}

pub mod update_notes_cmds {
    use clap::Args;

    #[derive(Clone, Debug, Args)]
    #[command(about = "Append a note")]
    pub struct Append {
        #[arg(long, help = "Note to append")]
        contents: String,
    }

    #[derive(Clone, Debug, Args)]
    #[command(about = "Replace contetns of a note")]
    pub struct Replace {
        #[arg(long, help = "Index to replace")]
        index: usize,

        #[arg(long, help = "Note that should be used for replacement")]
        contents: String,
    }

    #[derive(Clone, Debug, Args)]
    #[command(about = "Insert a note")]
    pub struct Insert {
        #[arg(long, help = "Index to insert note at")]
        index: usize,

        #[arg(long, help = "Note to insert")]
        contents: String,
    }

    #[derive(Clone, Copy, Debug, Args)]
    #[command(about = "Remove a note")]
    pub struct Remove {
        #[arg(long, help = "Index of note to remove")]
        index: usize,
    }

    #[derive(Clone, Copy, Debug, Args)]
    #[command(about = "Clear notes")]
    pub struct Clear;
}
