use clap::Args;

#[derive(Clone, Debug, Args)]
#[command(about = "Append a note")]
pub struct Append {
    #[arg(help = "Notes to append")]
    notes: Vec<String>,
}

impl Append {
    pub fn into_notes(self) -> Vec<String> {
        self.notes
    }
}

#[derive(Clone, Debug, Args)]
#[command(about = "Replace contetns of a note")]
pub struct Replace {
    #[arg(short, long, help = "Index to replace")]
    index: usize,

    #[arg(short, long, help = "Note that should be used for replacement")]
    contents: String,
}

impl Replace {
    pub fn into_parts(self) -> (usize, String) {
        (self.index, self.contents)
    }
}

#[derive(Clone, Debug, Args)]
#[command(about = "Insert a note")]
pub struct Insert {
    #[arg(short, long, help = "Index to insert note at")]
    index: usize,

    #[arg(short, long, help = "Note to insert")]
    contents: String,
}

impl Insert {
    pub fn into_parts(self) -> (usize, String) {
        (self.index, self.contents)
    }
}

#[derive(Clone, Copy, Debug, Args)]
#[command(about = "Remove a note")]
pub struct Remove {
    #[arg(help = "Index of note to remove")]
    index: usize,
}

impl Remove {
    pub fn index(&self) -> usize {
        self.index
    }
}

#[derive(Clone, Copy, Debug, Args)]
#[command(about = "Clear notes")]
pub struct Clear;
