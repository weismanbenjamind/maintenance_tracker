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
pub enum ContainersError {
    #[error("Failed to read maintenance log at path {path}. Error: {source}.")]
    FailedLoad {
        path: PathBuf,

        #[source]
        source: std::io::Error,
    },

    #[error("Failed to deserialize maintenance log from toml. Error: {0}.")]
    FailedDerserialize(#[from] toml::de::Error),
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
    NextServices(#[from] ServiceOptionsError),
}

#[derive(Debug, Error)]
pub enum ServiceOptionsError {
    #[error(
        "'miles_from_current' and 'current_miles' must both be 'Some' if one is set to 'Some'."
    )]
    InvalidMileageArgs,

    #[error("'today' can only be passed if 'months_from_today' is Some.")]
    InvalidDateArgs,
}
