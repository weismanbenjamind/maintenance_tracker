use clap::{Args, Subcommand};

#[derive(Clone, Debug, Args)]
#[command(about = "Append a note")]
pub(super) struct Append {
    #[arg(help = "Notes to append")]
    notes: Vec<String>,
}

impl Append {
    pub(super) fn into_notes(self) -> Vec<String> {
        self.notes
    }
}

#[derive(Clone, Debug, Args)]
#[command(about = "Replace contetns of a note")]
pub(super) struct Replace {
    #[arg(help = "Index to replace")]
    index: usize,

    #[arg(help = "Note that should be used for replacement")]
    contents: String,
}

impl Replace {
    pub(super) fn into_parts(self) -> (usize, String) {
        (self.index, self.contents)
    }
}

#[derive(Clone, Debug, Args)]
#[command(about = "Insert a note")]
pub(super) struct Insert {
    #[arg(help = "Index to insert note at")]
    index: usize,

    #[arg(help = "Note to insert")]
    contents: String,
}

impl Insert {
    pub(super) fn into_parts(self) -> (usize, String) {
        (self.index, self.contents)
    }
}

#[derive(Clone, Copy, Debug, Args)]
#[command(about = "Remove a note")]
pub(super) struct Remove {
    #[arg(help = "Index of note to remove")]
    index: usize,
}

impl Remove {
    pub(super) fn index(&self) -> usize {
        self.index
    }
}

#[derive(Clone, Copy, Debug, Args)]
#[command(about = "Clear notes")]
pub(super) struct Clear;

#[derive(Clone, Debug, Subcommand)]
pub(super) enum UpdateNotesCmd {
    Append(Append),
    Replace(Replace),
    Insert(Insert),
    Remove(Remove),
    Clear(Clear),
}
