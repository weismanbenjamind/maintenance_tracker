use clap::{Args, Subcommand};

use crate::containers::{MaintenanceLog, ServiceMetdata};
use crate::errors::CmdsError;
use log::info;

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

    // TODO - need a command to update the next service in here
    pub fn run(mut self, log: &mut MaintenanceLog) -> Result<(), CmdsError> {
        info!("Updating maintenance log");

        let mut metadata = log
            .remove(&self.id)
            .ok_or_else(|| CmdsError::IdNotFound(self.id().into()))?;

        // If want to update the id need to remove the current metadata and insert at the new id
        // Another option is to always remove then for the id cmd just update the self.id attribute to be the update
        // At the end just put the metadata back with the proper id

        match self.cmd {
            Cmd::Name(args) => metadata.name = args.name,
            Cmd::Id(args) => {
                if log.contains(&args.id) {
                    return Err(CmdsError::IdExists(args.id, metadata.name));
                }
                // Set the id attribute so when we insert back into the map we insert with the new id
                self.id = args.id
            }
            Cmd::MilesInterval(args) => metadata.set_service_interval_miles(args.miles),
            Cmd::MonthInterval(args) => metadata.set_service_interavl_months(args.months),
            Cmd::Notes(update_notes_cmd) => update_notes_cmd.run(&mut metadata),
            _ => todo!("Finish these up"),
        }

        // Don't need to check the option return since when we remove above we are sure this key does not exist
        // We also check during the update id cmd that they new id does not exist
        log.insert(&self.id, metadata);

        Ok(())
    }
}

#[derive(Clone, Debug, Subcommand)]
pub enum Cmd {
    Name(UpdateName),
    Id(UpdateId), // Do this last
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

    pub fn run(self) {}
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

    pub fn run(self) {}
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

    pub fn run(self) {}
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

    pub fn run(self) {}
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

    pub fn run(self, metadata: &mut ServiceMetdata) {
        match self.cmd {
            UpdateNotesCmd::Append(args) => metadata.exetend_notes(args.into_notes()),
            UpdateNotesCmd::Replace(args) => {
                let (index, contents) = args.into_parts();
                metadata.replace_note(index, &contents);
            }
            UpdateNotesCmd::Insert(args) => {
                let (index, contents) = args.into_parts();
                metadata.insert_note(index, &contents);
            }
            UpdateNotesCmd::Remove(args) => metadata.remove_note(args.index()),
            UpdateNotesCmd::Clear(_) => metadata.clear_notes(),
        }
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

    pub fn run(self) {}
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
        #[arg(help = "Notes to append")]
        notes: Vec<String>,
    }

    impl Append {
        pub fn new(notes: Vec<String>) -> Self {
            Self { notes }
        }

        pub fn notes(&self) -> &[String] {
            &self.notes
        }

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

        pub fn run(self) {}
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

        pub fn run(self) {}
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

        pub fn run(self) {}
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
