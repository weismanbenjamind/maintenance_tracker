use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MaintenanceTrackerError {
    #[error(transparent)]
    Containers(#[from] ContainersError),

    #[error(transparent)]
    Verbosity(#[from] VerbosityError),

    #[error(transparent)]
    CfgResolve(#[from] CfgResolveError),
}

#[derive(Debug, Error)]
pub enum ContainersError {
    #[error("Failed to read maintenance log. Error: {0}.")]
    FailedLoad(#[from] std::io::Error),

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
        "Failed to resolve path to config. Path {0} does not exist and environment variable {0} is not set."
    )]
    ResolveFailure(PathBuf, String),
}
