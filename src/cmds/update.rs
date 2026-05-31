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
    // TODO - need a command to update the next service in here
    pub fn run(mut self, log: &mut MaintenanceLog) -> Result<(), CmdsError> {
        info!("Updating maintenance log.");

        let mut metadata = match log.remove(&self.id) {
            Some(metadata) => metadata,
            None => return Err(CmdsError::IdNotFound(self.id)),
        };

        // If want to update the id need to remove the current metadata and insert at the new id
        // Another option is to always remove then for the id cmd just update the self.id attribute to be the update
        // At the end just put the metadata back with the proper id

        match self.cmd {
            Cmd::Name(args) => metadata.set_name(&args.name),
            Cmd::Id(args) => self.id = args.id, // Set the id attribute so when we insert back into the map we insert with the new id
            Cmd::MilesInterval(args) => metadata.set_service_interval_miles(args.miles),
            Cmd::MonthInterval(args) => metadata.set_service_interavl_months(args.months),
            Cmd::Notes(update_notes_cmd) => update_notes_cmd.run(&mut metadata),
            Cmd::Service(update_service_cmd) => update_service_cmd.run(&mut metadata)?,
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

#[derive(Clone, Debug, Args)]
#[command(about = "Update id")]
pub struct UpdateId {
    #[arg(help = "New id of service")]
    id: String,
}

#[derive(Clone, Copy, Debug, Args)]
#[command(about = "Update miles interval the service should be completed at")]
pub struct UpdateMilesInterval {
    #[arg(help = "New miles interval")]
    miles: u32,
}

#[derive(Clone, Copy, Debug, Args)]
#[command(about = "Update monthly interval the service should be completed at")]
pub struct MonthInterval {
    #[arg(help = "New monthly interval")]
    months: u32,
}

#[derive(Clone, Debug, Args)]
#[command(about = "Update notes")]
pub struct UpdateNotes {
    #[command(subcommand)]
    cmd: UpdateNotesCmd,
}

impl UpdateNotes {
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

#[derive(Clone, Debug, Subcommand)]
pub enum UpdateNotesCmd {
    Append(update_notes_cmds::Append),
    Replace(update_notes_cmds::Replace),
    Insert(update_notes_cmds::Insert),
    Remove(update_notes_cmds::Remove),
    Clear(update_notes_cmds::Clear),
}

#[derive(Clone, Copy, Debug, Args)]
#[command(about = "Update a previous service")]
pub struct UpdateService {
    #[command(subcommand)]
    cmd: UpdateServiceCmd,
}

impl UpdateService {
    pub fn run(self, metadata: &mut ServiceMetdata) -> Result<(), CmdsError> {
        match self.cmd {
            UpdateServiceCmd::Append(args) => {
                metadata.add_service_event(args.miles(), args.date());
                Ok(())
            }
            UpdateServiceCmd::Replace(args) => {
                metadata.replace_service_event(
                    args.curr_miles(),
                    args.curr_date(),
                    args.new_miles(),
                    args.new_date(),
                )?;
                Ok(())
            }
            UpdateServiceCmd::Remove(args) => {
                metadata.remove_service_event(args.miles(), args.date())?;
                Ok(())
            }
            UpdateServiceCmd::Clear(_) => {
                metadata.clear_previous_services();
                Ok(())
            }
        }
    }
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
        pub fn curr_miles(&self) -> Option<u32> {
            self.curr_service_specs.miles
        }

        pub fn curr_date(&self) -> Option<NaiveDate> {
            self.curr_service_specs.date
        }

        pub fn new_miles(&self) -> Option<u32> {
            self.updated_service_specs.miles()
        }

        pub fn new_date(&self) -> Option<NaiveDate> {
            self.updated_service_specs.date()
        }
    }

    #[derive(Clone, Copy, Debug, Args)]
    #[command(about = "Remove a service")]
    pub struct Remove {
        #[command(flatten)]
        service_specs: CurrServiceSpecs,
    }

    impl Remove {
        pub fn miles(&self) -> Option<u32> {
            self.service_specs.miles
        }

        pub fn date(&self) -> Option<NaiveDate> {
            self.service_specs.date
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

    #[derive(Clone, Copy, Debug, Args)]
    #[group(required = true, multiple = true)]
    pub struct UpdatedServiceSpecs {
        #[arg(short, long, help = "Mileage on vehicle to update service to")]
        new_miles: Option<u32>,

        #[arg(short = 'w', long, help = "Date to update service to")]
        new_date: Option<NaiveDate>,
    }

    impl UpdatedServiceSpecs {
        pub fn miles(&self) -> Option<u32> {
            self.new_miles
        }

        pub fn date(&self) -> Option<NaiveDate> {
            self.new_date
        }
    }
}
