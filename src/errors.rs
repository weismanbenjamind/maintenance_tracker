use chrono::ParseError;
use std::{
    num::ParseIntError,
    path::{Path, PathBuf},
};
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
pub enum ContainersError {
    #[error("Failed to read maintenance log at path {path}. Error: {source}.")]
    FailedLoad {
        path: PathBuf,

        #[source]
        source: std::io::Error,
    },

    #[error("Failed to write maintenance log to path {path}. Error: source {source}.")]
    FailedWrite {
        path: PathBuf,

        #[source]
        source: std::io::Error,
    },

    #[error("Failed to deserialize maintenance log from toml. Error: {0}.")]
    FailedDerserialize(#[from] toml::de::Error),

    #[error("Failed to serialize maintenance log to toml. Error: {0}.")]
    FailedSerialize(#[from] toml::ser::Error),
}

impl ContainersError {
    pub fn build_failed_write(e: std::io::Error, path: &Path) -> Self {
        Self::FailedWrite {
            path: path.into(),
            source: e,
        }
    }
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
    pub fn from_resolve_attempt(
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
    Diff(#[from] DiffOptionsError),

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

    #[error("Could not find service with id {0}")]
    IdNotFound(String),

    #[error("Id {0} already exists for maintenance item {0}")]
    IdExists(String, String),
}

#[derive(Debug, Error)]
pub enum DiffOptionsError {
    #[error(
        "'miles_from_current' and 'current_miles' must both be 'Some' if one is set to 'Some'."
    )]
    InvalidMileageArgs,

    #[error("'today' can only be passed if 'months_from_today' is Some.")]
    InvalidDateArgs,
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
    FailedWrite(#[source] std::io::Error),
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
