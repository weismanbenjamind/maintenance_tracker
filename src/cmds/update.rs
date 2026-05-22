use clap::{Args, Subcommand};

#[derive(Clone, Debug, Args)]
#[command(about = "Update a service")]
pub struct Update {
    #[arg(help = "ID of service to update")]
    id: String,

    #[command(subcommand)]
    cmd: Cmd,
}

impl Update {
    pub fn new(id: &str, cmd: Cmd) -> Self {
        Self { id: id.into(), cmd }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn cmd(&self) -> &Cmd {
        &self.cmd
    }
}

#[derive(Clone, Debug, Subcommand)]
pub enum Cmd {
    Name(UpdateName),
    Id(UpdateId),
    MilesInterval(UpdateMilesInterval),
    MonthInterval(MonthInterval),
    Notes(UpdateNotes),
    Service(UpdateService),
}

#[derive(Clone, Debug, Args)]
#[command(about = "Update name")]
pub struct UpdateName {
    #[arg(help = "New name of service")]
    name: String,
}

impl UpdateName {
    pub fn new(name: &str) -> Self {
        Self { name: name.into() }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug, Args)]
#[command(about = "Update id")]
pub struct UpdateId {
    #[arg(help = "New id of service")]
    id: String,
}

impl UpdateId {
    pub fn new(id: &str) -> Self {
        Self { id: id.into() }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Clone, Copy, Debug, Args)]
#[command(about = "Update miles interval the service should be completed at")]
pub struct UpdateMilesInterval {
    #[arg(help = "New miles interval")]
    miles: u32,
}

impl UpdateMilesInterval {
    pub fn new(miles: u32) -> Self {
        Self { miles }
    }

    pub fn miles(&self) -> u32 {
        self.miles
    }
}

#[derive(Clone, Copy, Debug, Args)]
#[command(about = "Update monthly interval the service should be completed at")]
pub struct MonthInterval {
    #[arg(help = "New monthly interval")]
    months: u32,
}

impl MonthInterval {
    pub fn new(months: u32) -> Self {
        Self { months }
    }

    pub fn months(&self) -> u32 {
        self.months
    }
}

#[derive(Clone, Debug, Args)]
#[command(about = "Update notes")]
pub struct UpdateNotes {
    #[command(subcommand)]
    cmd: UpdateNotesCmd,
}

impl UpdateNotes {
    pub fn new(cmd: UpdateNotesCmd) -> Self {
        Self { cmd }
    }

    pub fn cmd(&self) -> &UpdateNotesCmd {
        &self.cmd
    }
}

#[derive(Clone, Copy, Debug, Args)]
#[command(about = "Update a previous service")]
pub struct UpdateService {
    #[command(subcommand)]
    cmd: UpdateServiceCmd,
}

impl UpdateService {
    pub fn new(cmd: UpdateServiceCmd) -> Self {
        Self { cmd }
    }

    pub fn cmd(&self) -> &UpdateServiceCmd {
        &self.cmd
    }
}

#[derive(Clone, Debug, Subcommand)]
pub enum UpdateNotesCmd {
    Append(update_notes_cmds::Append),
    Replace(update_notes_cmds::Replace),
    Insert(update_notes_cmds::Insert),
    Remove(update_notes_cmds::Remove),
    Clear(update_notes_cmds::Clear),
}

#[derive(Clone, Copy, Debug, Subcommand)]
pub enum UpdateServiceCmd {
    Append(update_service_cmds::Append),
    Replace(update_service_cmds::Replace),
    Remove(update_service_cmds::Remove),
    Clear(update_service_cmds::Clear),
}

mod update_notes_cmds {
    use clap::Args;

    #[derive(Clone, Debug, Args)]
    #[command(about = "Append a note")]
    pub struct Append {
        #[arg(long, short, help = "Notes to append")]
        notes: Vec<String>,
    }

    impl Append {
        pub fn new(notes: Vec<String>) -> Self {
            Self { notes }
        }

        pub fn notes(&self) -> &[String] {
            &self.notes
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
        pub fn new(index: usize, contents: &str) -> Self {
            Self {
                index,
                contents: contents.into(),
            }
        }

        pub fn index(&self) -> usize {
            self.index
        }

        pub fn contents(&self) -> &str {
            &self.contents
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
        pub fn new(index: usize, contents: &str) -> Self {
            Self {
                index,
                contents: contents.into(),
            }
        }

        pub fn index(&self) -> usize {
            self.index
        }

        pub fn contents(&self) -> &str {
            &self.contents
        }
    }

    #[derive(Clone, Copy, Debug, Args)]
    #[command(about = "Remove a note")]
    pub struct Remove {
        #[arg(help = "Index of note to remove")]
        index: usize,
    }

    impl Remove {
        pub fn new(index: usize) -> Self {
            Self { index }
        }

        pub fn index(&self) -> usize {
            self.index
        }
    }

    #[derive(Clone, Copy, Debug, Args)]
    #[command(about = "Clear notes")]
    pub struct Clear;
}

mod update_service_cmds {
    use chrono::Local;
    use chrono::NaiveDate;
    use clap::Args;

    #[derive(Clone, Copy, Debug, Args)]
    #[command(about = "Append a service")]
    pub struct Append {
        #[arg(short, long, help = "Mileage of service")]
        miles: u32,

        #[arg(short, long, default_value_t = Local::now().date_naive(), help = "Date of service")]
        date: NaiveDate,
    }

    impl Append {
        pub fn new(miles: u32, date: NaiveDate) -> Self {
            Self { miles, date }
        }

        pub fn miles(&self) -> u32 {
            self.miles
        }

        pub fn date(&self) -> NaiveDate {
            self.date
        }
    }

    #[derive(Clone, Copy, Debug, Args)]
    #[command(about = "Replace a service")]
    pub struct Replace {
        #[command(flatten)]
        curr_service_specs: CurrServiceSpecs,

        #[command(flatten)]
        updated_service_specs: UpdatedServiceSpecs,
    }

    impl Replace {
        pub fn new(
            curr_service_specs: CurrServiceSpecs,
            updated_service_specs: UpdatedServiceSpecs,
        ) -> Self {
            Self {
                curr_service_specs,
                updated_service_specs,
            }
        }

        pub fn curr_service_specs(&self) -> CurrServiceSpecs {
            self.curr_service_specs
        }

        pub fn updated_service_specs(&self) -> UpdatedServiceSpecs {
            self.updated_service_specs
        }
    }

    #[derive(Clone, Copy, Debug, Args)]
    #[command(about = "Remove a service")]
    pub struct Remove {
        #[command(flatten)]
        service_specs: CurrServiceSpecs,
    }

    impl Remove {
        pub fn new(service_specs: CurrServiceSpecs) -> Self {
            Self { service_specs }
        }

        pub fn service_specs(&self) -> CurrServiceSpecs {
            self.service_specs
        }
    }

    #[derive(Clone, Copy, Debug, Args)]
    #[command(about = "Clear all services")]
    pub struct Clear;

    #[derive(Clone, Copy, Debug, Args)]
    #[group(required = true, multiple = true)]
    pub struct CurrServiceSpecs {
        #[arg(short, long, help = "Mileage on vehicle when service was performed")]
        miles: Option<u32>,

        #[arg(short, long, help = "Date when service was performed")]
        date: Option<NaiveDate>,
    }

    impl CurrServiceSpecs {
        pub fn new(miles: Option<u32>, date: Option<NaiveDate>) -> Self {
            Self { miles, date }
        }

        pub fn miles(&self) -> Option<u32> {
            self.miles
        }

        pub fn date(&self) -> Option<NaiveDate> {
            self.date
        }
    }

    #[derive(Clone, Copy, Debug, Args)]
    #[group(required = true, multiple = true)]
    pub struct UpdatedServiceSpecs {
        #[arg(short, long, help = "Mileage on vehicle to update service to")]
        new_miles: Option<u32>,

        #[arg(short = 'w', long, help = "Date to update service to")]
        new_date: Option<NaiveDate>,
    }

    impl UpdatedServiceSpecs {
        pub fn new(new_miles: Option<u32>, new_date: Option<NaiveDate>) -> Self {
            Self {
                new_miles,
                new_date,
            }
        }

        pub fn new_miles(&self) -> Option<u32> {
            self.new_miles
        }

        pub fn new_date(&self) -> Option<NaiveDate> {
            self.new_date
        }
    }
}
