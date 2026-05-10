use crate::errors::VerbosityError;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt as tracing_subscriber_fmt;

const RUST_LOG: &str = "RUST_LOG";
const WARN: &str = "warn";
const INFO: &str = "info";
const DEBUG: &str = "debug";

pub fn set_verbosity(verbosity: u8) -> Result<(), VerbosityError> {
    let log_level = get_log_level(verbosity);
    init_tracing_subscriber(&log_level)
}

fn get_log_level(verbosity: u8) -> String {
    match std::env::var(RUST_LOG) {
        Ok(level) => level,
        Err(_) => match verbosity {
            0 => WARN,
            1 => INFO,
            _ => DEBUG,
        }
        .to_string(),
    }
}

// Note - below in a true binary crate we just panic with try_init().expect!("...")
// For binary crate if it's something we can't contril (tracing init) then just panic
// If it is something we can contraol (e.g. pointed to incorrect file path) then bubble up
// For library crates - never panic. Someone else is using the code we can't panic and crash their code
// Not panicing below for practice writing a library
fn init_tracing_subscriber(log_level: &str) -> Result<(), VerbosityError> {
    tracing_subscriber_fmt()
        .without_time()
        .with_env_filter(EnvFilter::new(log_level))
        .try_init()
        .map_err(|e| VerbosityError::FailedInitialization(e.to_string()))
}
