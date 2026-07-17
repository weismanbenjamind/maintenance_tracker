//! # Errors
//!
//! Module to house all errors from maintenance_tracker library

use chrono::ParseError;
use std::num::ParseIntError;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Top level error coming out of the MaintenanceTracker library.
#[derive(Error, Debug)]
pub enum MaintenanceTrackerError {
    #[error(transparent)]
    Containers(#[from] ContainersError),

    #[error(transparent)]
    CfgResolve(#[from] CfgResolveError),

    #[error(transparent)]
    Cmds(#[from] CmdsError),

    /// Error to be raised if the MaintenanceTracker winds up in an invalid state.
    /// Typically not reachable due to type system, especially for args are coming from clap.
    #[error("{0}")]
    InvalidState(String),
}

/// Error to be raised when resolving the path to the configuration file housing maintenance records.
#[derive(Debug, Error)]
pub enum CfgResolveError {
    /// Error for when the path to the config could not be found
    /// an no attempt is being made to resolve the path from environment variables.
    #[error(
        "Failed to resolve path to config. Path {0} does not exist and skipping environment variable resolution."
    )]
    ResolveFailure(PathBuf),

    /// Error for when the path to the config could not be found
    /// including when trying to use environment variables to resolve the path.
    #[error(
        "Failed to resolve path to config. Path {0} does not exist and environment variable {1} is not set."
    )]
    ResolveFailureEnv(PathBuf, String),
}

impl CfgResolveError {
    /// Build a CfgResolveError when attempting to resolve the config path.
    /// Chooses the right error type based on if a specific value is passed to skip
    /// resolving the path from environment variables.
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

/// Top level error for all CLI commands.
#[derive(Debug, Error)]
pub enum CmdsError {
    #[error(transparent)]
    Init(#[from] InitError),

    #[error(transparent)]
    Containers(#[from] ContainersError),

    #[error(transparent)]
    Update(#[from] UpdateError),

    #[error(transparent)]
    Diff(#[from] DiffError),

    #[error(transparent)]
    IdNotFound(#[from] IdNotFoundError),

    #[error(transparent)]
    MaintenanceLog(#[from] MaintenanceLogError),

    /// Error to be raised if attempting to initialize a service item with an id that already exists
    #[error("Id {0} already exists for maintenance item {0}")]
    IdExists(String, String),
}

impl CmdsError {
    /// Builds a new CmdsError::IdNotFound error.
    pub(crate) fn new_id_not_found(id: &str) -> Self {
        Self::IdNotFound(IdNotFoundError::IdNotFound(id.into()))
    }
}

/// Top level errors for all container related operations.
#[derive(Debug, Error)]
pub enum ContainersError {
    #[error(transparent)]
    MaintenanceLog(#[from] MaintenanceLogError),

    #[error(transparent)]
    Notes(#[from] NotesError),
}

/// Top level errors for all Maintenance Log related operations.
#[derive(Debug, Error)]
pub enum MaintenanceLogError {
    /// Error for when the maintenance log could not be read into memory.
    #[error("Failed to read maintenance log at path {0}. Error: {1}.")]
    FailedLoad(PathBuf, #[source] std::io::Error),

    /// Error for when the maintenance log could not be serialized to disk.
    #[error("Failed to write maintenance log to path {0}. Error: {1}.")]
    FailedWrite(PathBuf, #[source] std::io::Error),

    /// Error for when the maintenance log could not be deserialized to .toml format.
    #[error("Failed to deserialize maintenance log from toml. Error: {0}.")]
    FailedDerserialize(#[from] toml::de::Error),

    /// Error for when the maintenance log could not be serialized from .toml format.
    #[error("Failed to serialize maintenance log to toml. Error: {0}.")]
    FailedSerialize(#[from] toml::ser::Error),
}

impl MaintenanceLogError {
    /// Build a new MaintenanceLogError::FailedWrite error.
    pub(crate) fn new_failed_write(e: std::io::Error, path: &Path) -> Self {
        Self::FailedWrite(path.into(), e)
    }
}

/// Error dealing with Ids not found in the maintenance log.
#[derive(Debug, Error)]
pub enum IdNotFoundError {
    /// Error to be raised when an Id was not found in the maintenance log.
    #[error("Could not find service with id {0}")]
    IdNotFound(String),
}

/// Error for dealing with note operations.
#[derive(Debug, Error)]
pub enum NotesError {
    /// Error for when a note is not found at the expected index.
    #[error("Note not found at index {0}.")]
    NoteIndexNotFound(usize),

    /// Error for when a note is requested at an index that does not exist.
    #[error("Note index {0} out of range. {1} notes present.")]
    NoteIndexOutOfRange(usize, usize),

    /// Error for when notes are not set.
    #[error("No notes present.")]
    NotesNotSet,
}

/// Error for dealing with operating on previous services.
#[derive(Debug, Error)]
pub enum PreviousServicesError {
    /// Error for when the previous services are note set.
    #[error("No previous services present.")]
    PreviousServicesNotSet,

    /// Error for when a requested previous service could not be found.
    #[error("Could not find target service event.")]
    ServiceEventNotFound,

    /// Error for when multiple service events were found for the same metrics.
    #[error("Found multiple service events.")]
    MultipleServiceEvents,

    /// Error for when optional args aren't passed correctly.
    /// Clap should handle this error at app boundary in practice.
    #[error("At least one of notes or date must be set.")]
    InvalidOptionalArgs,
}

/// Error for dealing with initialization of maintenance items.
#[derive(Debug, Error)]
pub enum InitError {
    /// Error for when previous service is not in correct format when parsing from a String.
    #[error(
        "Failed to parse --previous-service (-p) argument. Ensure all previous service arguments are in the format 'miles;YYYY-MM-DD'."
    )]
    FailedPreviousServiceParse,

    /// Error for when parsing miles from a String into a u32.
    #[error("Could not parse miles into u32. Error: {0}")]
    InvalidMilesFormat(#[from] ParseIntError),

    /// Error for when parsing date from a String into a chrono::NaiveDate.
    #[error("Could not parse date. Ensure date is in format YYYY-MM-DD. Error: {0}")]
    InvalidDateFormat(#[from] ParseError),
}

/// Error for when performing update operations.
#[derive(Debug, Error)]
pub enum UpdateError {
    /// Error for when a note update operation fails.
    #[error("Failed note update operation. Error: {0}")]
    FailedNoteUpdate(#[from] NotesError),

    /// Error for when a previous service update operation fails.
    #[error("Failed service update operation. Error: {0}")]
    FailedServiceUpdate(#[from] PreviousServicesError),

    /// Error for when args are invalid for updating next service.
    /// In practice Clap should enforce this error at the app boundary.
    #[error("Must pass one of miles or date when updating next service")]
    UpdateNextServiceArgs,

    /// Error for if current service identifiers for an update event winds up in an invalid state.
    /// Type system and Clap should enforce that this error nerver gets hit.
    #[error(
        "Invalid state for update operation. One or both of current miles/date must be passed to identify the service"
    )]
    InvalidCurrentServiceIds,

    /// Error for if update service args for an update event winds up in an invalid state.
    /// Type system and Clap should enforce that this error nerver gets hit.
    #[error(
        "Invalid state for update operation. One or both of new mils/new date must be passed to update a service"
    )]
    InvalidServiceUpdateArgs,

    /// Error for if args for an update event winds up in an invalid state.
    /// Type system and Clap should enforce that this error nerver gets hit.
    #[error(
        "Invalid state for update operation. One or both of miles/months must be passed to update the service interval"
    )]
    InvalidServiceIntervalUpdateArgs,
}

/// Error for diff operations.
#[derive(Debug, Error)]
pub enum DiffError {
    /// Error for if threshold command args are not passed correctly.
    /// Clap should enforce this error never gets hit in practice.
    #[error(
        "Both mileage and curr mileage and/or a date must be present for a threshold diff operation."
    )]
    InvalidThresholdArgs,

    /// Error for if interval command args are not passed correctly.
    /// Clap should enforce this error never gets hit in practice.
    #[error("Must pass miles and current miles and/or months for an interval diff operation.")]
    InvalidIntervalArgs,
}
