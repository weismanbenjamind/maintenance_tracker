use clap::{Args, Subcommand};

#[derive(Clone, Debug, Args)]
#[command(about = "Update a service")]
pub struct Update {
    #[arg(short, long, help = "ID of service to update")]
    id: String,

    #[command(subcommand)]
    command: Cmd,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Cmd {
    Name(UpdateName),
    Id(UpdateId),
    MilesInterval(UpdateMilesInterval),
    MonthlyInterval(MonthInterval),
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

#[derive(Clone, Copy, Debug, Args)]
#[command(about = "Update a previous service")]
pub struct UpdateService {
    #[command(subcommand)]
    cmd: UpdateServiceCmd,
}

#[derive(Clone, Debug, Subcommand)]
pub enum UpdateNotesCmd {
    Append(update_notes_cmd::Append),
    Replace(update_notes_cmd::Replace),
    Insert(update_notes_cmd::Insert),
    Remove(update_notes_cmd::Remove),
    Clear(update_notes_cmd::Clear),
}

#[derive(Clone, Copy, Debug, Subcommand)]
pub enum UpdateServiceCmd {
    Append(update_service_cmd::Append),
    Replace(update_service_cmd::Replace),
    Remove(update_service_cmd::Remove),
    Clear(update_service_cmd::Clear),
}

pub mod update_notes_cmd {
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

pub mod update_service_cmd {
    use chrono::Local;
    use chrono::NaiveDate;
    use clap::Args;

    #[derive(Clone, Copy, Debug, Args)]
    #[command(about = "Append a service")]
    pub struct Append {
        #[arg(long, help = "Mileage of service")]
        miles: u32,

        #[arg(long, default_value_t = Local::now().date_naive(), help = "Date of service")]
        date: NaiveDate,
    }

    #[derive(Clone, Copy, Debug, Args)]
    #[command(about = "Replace a service")]
    pub struct Replace {
        #[command(flatten)]
        curr_service_specs: CurrServiceSpecs,

        #[command(flatten)]
        updated_service_specs: UpdatedServiceSpecs,
    }

    #[derive(Clone, Copy, Debug, Args)]
    #[command(about = "Remove a service")]
    pub struct Remove {
        #[command(flatten)]
        service_specs: CurrServiceSpecs,
    }

    #[derive(Clone, Copy, Debug, Args)]
    #[command(about = "Clear all services")]
    pub struct Clear;

    #[derive(Clone, Copy, Debug, Args)]
    #[group(required = true, multiple = true)]
    pub struct CurrServiceSpecs {
        #[arg(long, help = "Mileage on vehicle when service was performed")]
        miles: Option<u32>,

        #[arg(long, help = "Date when service was performed")]
        date: Option<NaiveDate>,
    }

    #[derive(Clone, Copy, Debug, Args)]
    #[group(required = true, multiple = true)]
    pub struct UpdatedServiceSpecs {
        #[arg(long, help = "Mileage on vehicle to update service to")]
        new_miles: Option<u32>,

        #[arg(long, help = "Date to update service to")]
        new_date: Option<NaiveDate>,
    }
}
