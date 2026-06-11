use chrono::ParseError;
use std::num::ParseIntError;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MaintenanceTrackerError {
    #[error(transparent)]
    Containers(#[from] ContainersError),

    #[error(transparent)]
    Verbosity(#[from] VerbosityError),

    #[error(transparent)]
    CfgResolve(#[from] CfgResolveError),

    #[error(transparent)]
    Cmds(#[from] CmdsError),
}

#[derive(Debug, Error)]
pub enum VerbosityError {
    // Must use normal error below
    // try_init() off tracing subscriber returns a Box<dyn ...>
    // At this point just grab the string error message and lose error chain
    #[error("Failed to initialize tracing subscriber. Error: {0}.")]
    FailedInitialization(String),
}

#[derive(Debug, Error)]
pub enum CfgResolveError {
    #[error(
        "Failed to resolve path to config. Path {0} does not exist and skipping environment variable resolution."
    )]
    ResolveFailure(PathBuf),

    #[error(
        "Failed to resolve path to config. Path {0} does not exist and environment variable {1} is not set."
    )]
    ResolveFailureEnv(PathBuf, String),
}

impl CfgResolveError {
    pub(crate) fn from_resolve_attempt(
        maintenance_log: &Path,
        maintenance_log_env: &str,
        skip_env: &str,
    ) -> Self {
        match maintenance_log_env == skip_env {
            true => Self::ResolveFailure(maintenance_log.into()),
            false => Self::ResolveFailureEnv(maintenance_log.into(), maintenance_log_env.into()),
        }
    }
}

#[derive(Debug, Error)]
pub enum CmdsError {
    #[error(transparent)]
    Init(#[from] InitError),

    #[error(transparent)]
    List(#[from] ListError),

    #[error(transparent)]
    Next(#[from] NextError),

    #[error(transparent)]
    Detail(#[from] DetailError),

    #[error(transparent)]
    Complete(#[from] CompleteError),

    #[error(transparent)]
    Containers(#[from] ContainersError),

    #[error(transparent)]
    Update(#[from] UpdateError),

    #[error(transparent)]
    Diff(#[from] DiffError),

    #[error(transparent)]
    Status(#[from] StatusError),

    #[error(transparent)]
    IdNotFound(#[from] IdNotFoundError),

    #[error("Id {0} already exists for maintenance item {0}")]
    IdExists(String, String),
}

impl CmdsError {
    pub(crate) fn new_id_not_found(id: &str) -> Self {
        Self::IdNotFound(IdNotFoundError::IdNotFound(id.into()))
    }
}

#[derive(Debug, Error)]
pub enum ContainersError {
    #[error(transparent)]
    MaintenanceLog(#[from] MaintenanceLogError),

    #[error(transparent)]
    Notes(#[from] NotesError),
}

#[derive(Debug, Error)]
pub enum MaintenanceLogError {
    #[error("Failed to read maintenance log at path {0}. Error: {1}.")]
    FailedLoad(PathBuf, #[source] std::io::Error),

    #[error("Failed to write maintenance log to path {0}. Error: {1}.")]
    FailedWrite(PathBuf, #[source] std::io::Error),

    #[error("Failed to deserialize maintenance log from toml. Error: {0}.")]
    FailedDerserialize(#[from] toml::de::Error),

    #[error("Failed to serialize maintenance log to toml. Error: {0}.")]
    FailedSerialize(#[from] toml::ser::Error),
    // #[error(transparent)]
    // IdNotFound(#[from] IdNotFoundError),
}

impl MaintenanceLogError {
    pub(crate) fn new_failed_write(e: std::io::Error, path: &Path) -> Self {
        Self::FailedWrite(path.into(), e)
    }
}

#[derive(Debug, Error)]
pub enum IdNotFoundError {
    #[error("Could not find service with id {0}")]
    IdNotFound(String),
}

#[derive(Debug, Error)]
pub enum NotesError {
    #[error("Note not found at index {0}.")]
    NoteIndexNotFound(usize),

    #[error("Note index {0} out of range. {1} notes present.")]
    NoteIndexOutOfRange(usize, usize),

    #[error("No notes present.")]
    NotesNotSet,
}

#[derive(Debug, Error)]
pub enum PreviousServicesError {
    #[error("No previous services present.")]
    PreviousServicesNotSet,

    #[error("Could not find target service event.")]
    ServiceEventNotFound,

    #[error("Found multiple service events.")]
    MultipleServiceEvents,

    #[error("At least one of notes or date must be set.")]
    InvalidOptionalArgs,
}

#[derive(Debug, Error)]
pub enum InitError {
    #[error(
        "Failed to parse --previous-service (-p) argument. Ensure all previous service arguments are in the format 'miles;YYYY-MM-DD'."
    )]
    FailedPreviousServiceParse,

    #[error("Could not parse miles into u32. Error: {0}")]
    InvalidMilesFormat(#[from] ParseIntError),

    #[error("Could not parse date. Ensure date is in format YYYY-MM-DD. Error: {0}")]
    InvalidDateFormat(#[from] ParseError),

    #[error("Failed to write initialized service details. Error {0}")]
    FailedWrite(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum ListError {
    #[error("Failed to list service ids. Error: {0}")]
    FailedList(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum NextError {
    #[error("Failed to write next service for id {0}. Error: {1}")]
    FailedWrite(String, #[source] std::io::Error),
}

#[derive(Debug, Error)]
pub enum DetailError {
    #[error("Failed to write details for service with id: {0}. Error: {1}")]
    FailedWrite(String, #[source] std::io::Error),
}

#[derive(Debug, Error)]
pub enum CompleteError {
    #[error("Failed to write details for service completion for service with id {0}. Error: {1}")]
    FailedWrite(String, #[source] std::io::Error),
}

#[derive(Debug, Error)]
pub enum UpdateError {
    #[error("Failed note update operation. Error: {0}")]
    FailedNoteUpdate(#[from] NotesError),

    #[error("Failed service update operation. Error: {0}")]
    FailedServiceUpdate(#[from] PreviousServicesError),
}

#[derive(Debug, Error)]
pub enum DiffError {
    #[error(
        "Both mileage and curr mileage and/or a date must be present for a threshold diff operation."
    )]
    InvalidThresholdArgs,

    #[error("Must pass miles and current miles and/or months for an interval diff operation.")]
    InvalidIntervalArgs,

    #[error("Failed to write Diff result to stdout. Error: {0}")]
    FailedStdOutWrite(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum StatusError {
    #[error("Failed to write to stdout. Error: {0}")]
    FailedStdoutWrite(#[from] std::io::Error),
}
