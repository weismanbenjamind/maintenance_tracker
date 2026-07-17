//! # Verbosity
//!
//! # Module to house functionality for setting verbosit for CLI runs.

use std::sync::OnceLock;

use log::info;
use std::fmt::Write;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt as tracing_subscriber_fmt;

static LOG_LEVEL: OnceLock<String> = OnceLock::new();
const WARN: &str = "warn";
const INFO: &str = "info";
const DEBUG: &str = "debug";

/// Sets verbosity for CLI run.
/// The verbosity arg maps to...
/// - 'warn' if zero is passed
/// - 'info' if 1 is passed
/// - 'debug' if anything greater than 1 is passed
///
/// If the rust_log argument is passed it will be used instead of the verbosity arg.
/// The rust log arg should be one of th valid values from the `RUST_LOG` environment variable.
pub(crate) fn set_verbosity(verbosity: u8, rust_log: Option<String>) {
    let log_level = get_log_level(verbosity, rust_log);
    init_tracing_subscriber(&log_level);
    _ = LOG_LEVEL.set(log_level);
}

/// Gets the log level for a given verbosity and rust_log variable.
/// The verbosity arg maps to...
/// - 'warn' if zero is passed
/// - 'info' if 1 is passed
/// - 'debug' if anything greater than 1 is passed
///
/// If the rust_log argument is passed it will be used instead of the verbosity arg.
/// The rust log arg should be one of th valid values from the `RUST_LOG` environment variable.
fn get_log_level(verbosity: u8, rust_log: Option<String>) -> String {
    match rust_log {
        Some(level) => level,
        None => match verbosity {
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
/// Initializes a tracing subscriber from the `tracing_subscriber` crate at the given log level.
fn init_tracing_subscriber(log_level: &str) {
    tracing_subscriber_fmt()
        .with_env_filter(EnvFilter::new(log_level))
        .try_init()
        .unwrap_or_else(|_| {
            let mut buf = String::new();
            _ = write!(buf, "Verbosity already set");
            if let Some(log_level) = LOG_LEVEL.get() {
                _ = write!(buf, " to {log_level}");
            }
            _ = write!(buf, ". Skipping vebosity initialization.");
            info!("{buf}");
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verbosity_get_log_level_warn() {
        assert_eq!(get_log_level(0, None), "warn");
    }

    #[test]
    fn test_verbosity_get_log_level_info() {
        assert_eq!(get_log_level(1, None), "info");
    }

    #[test]
    fn test_verbosity_get_log_level_debug_2() {
        assert_eq!(get_log_level(2, None), "debug");
    }

    #[test]
    fn test_verbosity_get_log_level_debug_26() {
        assert_eq!(get_log_level(26, None), "debug");
    }

    #[test]
    fn test_verbosity_pass_rust_log() {
        assert_eq!(get_log_level(26, Some("warn".to_string())), "warn");
    }
}
