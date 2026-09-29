//! # Cfg Resolve
//!
//! Module housing utilites to resolve the path to the config

use std::path::{Path, PathBuf};

use log::info;

use crate::errors::CfgResolveError;

const SKIP_ENV_FLAG: &str = "skip";

/// Resolve the path to the maintenance log config.
/// Will Return `Ok(maintenance_log)` if the path exists.
/// Otherwise will attempt to read `maintenance_log_env` from the environment
/// and use this value for the path to the maintenance log config.
/// If `skip` is passed for `maintenance_log_env` the value will not be attempted
/// to be read from the environment.
pub(crate) fn resolve_cfg(
    maintenance_log: &Path,
    maintenance_log_env: &str,
) -> Result<PathBuf, CfgResolveError> {
    let result = match maintenance_log.exists() {
        true => Ok(maintenance_log.into()),
        false => resolve_from_env(maintenance_log_env).ok_or_else(|| {
            CfgResolveError::from_resolve_attempt(
                maintenance_log,
                maintenance_log_env,
                SKIP_ENV_FLAG,
            )
        }),
    };

    if let Ok(path) = &result {
        info!("Resolved maintenance log path to {}.", path.display())
    }

    result
}

/// Resolves the path to the mantenance log config from the environment.
/// Will attempt to read `maintenance_log_env` from the environment
/// and use this value for the path to the maintenance log config.
/// If `skip` is passed for `maintenance_log_env` the value will not be attempted
/// to be read from the environment.
fn resolve_from_env(maintenance_log_env: &str) -> Option<PathBuf> {
    match maintenance_log_env == SKIP_ENV_FLAG {
        true => None,
        false => std::env::var(maintenance_log_env).ok().map(PathBuf::from),
    }
}

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn test_cfg_resolve_resolve_cfg_path_exists_skip_env() {
        let tempfile = tempfile::NamedTempFile::new().unwrap();
        let path = tempfile.path();
        assert_eq!(resolve_cfg(path, "skip").unwrap(), path.to_owned());
    }

    // THIS TEST WILL ATTEMPT TO READ THE ENVIRONMENT
    // ENVIRONMENT VARIABLES ARE PROCESS WIDE ACROSS THREADS
    // IF TWO THREADS ATTEMPT TO SET/UNSET maintenance_log_env
    // THEY WILL INTERFERE WITH EACH OTHER
    // RUN cargo test -- --test-threads=1 TO AVOID RACE CONDITIONS
    // LIKE THE ONE JUST DESCRIBED
    #[test]
    fn test_cfg_resolve_resolve_cfg_path_exists_env_var_path_not_exists() {
        let tempfile_1 = tempfile::NamedTempFile::new().unwrap();
        let path_1 = tempfile_1.path();

        let path_2 = Path::new("does/not/exist");
        assert!(!path_2.exists());

        // Try to make the below env var unique to avoid race conditions between threads
        let maintenance_log_env = "MAINTENANCE_LOG_NOT_PRESENT_1";
        let value = path_2.as_os_str();

        temp_env::with_var(maintenance_log_env, Some(value), || {
            let found = resolve_cfg(path_1, maintenance_log_env).unwrap();
            assert_eq!(found, path_1.to_owned());
        })
    }

    // THIS TEST WILL ATTEMPT TO READ THE ENVIRONMENT
    // ENVIRONMENT VARIABLES ARE PROCESS WIDE ACROSS THREADS
    // IF TWO THREADS ATTEMPT TO SET/UNSET maintenance_log_env
    // THEY WILL INTERFERE WITH EACH OTHER
    // RUN cargo test -- --test-threads=1 TO AVOID RACE CONDITIONS
    // LIKE THE ONE JUST DESCRIBED
    #[test]
    fn test_cfg_resolve_resolve_cfg_path_exists_env_var_path_exists() {
        let tempfile_1 = tempfile::NamedTempFile::new().unwrap();
        let path_1 = tempfile_1.path();

        let tempfile_2 = tempfile::NamedTempFile::new().unwrap();
        let path_2 = tempfile_2.path();

        // Try to make the below env var unique to avoid race conditions between threads
        let maintenance_log_env = "MAINTENANCE_LOG_PRESENT_1";
        let value = path_2.as_os_str();

        temp_env::with_var(maintenance_log_env, Some(value), || {
            let found = resolve_cfg(path_1, maintenance_log_env).unwrap();
            assert_eq!(found, path_1.to_owned());
        })
    }

    // THIS TEST WILL ATTEMPT TO READ THE ENVIRONMENT
    // ENVIRONMENT VARIABLES ARE PROCESS WIDE ACROSS THREADS
    // IF TWO THREADS ATTEMPT TO SET/UNSET maintenance_log_env
    // THEY WILL INTERFERE WITH EACH OTHER
    // RUN cargo test -- --test-threads=1 TO AVOID RACE CONDITIONS
    // LIKE THE ONE JUST DESCRIBED
    #[test]
    fn test_cfg_resolve_resolve_cfg_path_exists_env_var_not_set() {
        let tempfile_1 = tempfile::NamedTempFile::new().unwrap();
        let path_1 = tempfile_1.path();

        // Try to make the below env var unique to avoid race conditions between threads
        let maintenance_log_env = "MAINTENANCE_LOG_NOT_PRESENT_2";
        let value: Option<&str> = None;

        temp_env::with_var(maintenance_log_env, value, || {
            let found = resolve_cfg(path_1, maintenance_log_env).unwrap();
            assert_eq!(found, path_1.to_owned());
        })
    }

    // THIS TEST WILL ATTEMPT TO READ THE ENVIRONMENT
    // ENVIRONMENT VARIABLES ARE PROCESS WIDE ACROSS THREADS
    // IF TWO THREADS ATTEMPT TO SET/UNSET maintenance_log_env
    // THEY WILL INTERFERE WITH EACH OTHER
    // RUN cargo test -- --test-threads=1 TO AVOID RACE CONDITIONS
    // LIKE THE ONE JUST DESCRIBED
    #[test]
    fn test_cfg_resolve_resolve_cfg_path_not_exists_env_var_path_not_exists() {
        let path_1 = Path::new("some_path.txt");
        assert!(!path_1.exists());

        let path_2 = Path::new("some_other_path.txt");
        assert!(!path_2.exists());

        // Try to make the below env var unique to avoid race conditions between threads
        let maintenance_log_env = "MAINTENANCE_LOG_PRESENT_2";
        let value = path_2.as_os_str();

        temp_env::with_var(maintenance_log_env, Some(value), || {
            let found = resolve_cfg(path_1, maintenance_log_env).unwrap();
            assert_eq!(found, path_2.to_owned());
        })
    }

    // THIS TEST WILL ATTEMPT TO READ THE ENVIRONMENT
    // ENVIRONMENT VARIABLES ARE PROCESS WIDE ACROSS THREADS
    // IF TWO THREADS ATTEMPT TO SET/UNSET maintenance_log_env
    // THEY WILL INTERFERE WITH EACH OTHER
    // RUN cargo test -- --test-threads=1 TO AVOID RACE CONDITIONS
    // LIKE THE ONE JUST DESCRIBED
    #[test]
    fn test_cfg_resolve_resolve_cfg_path_not_exists_env_var_path_exists() {
        let path_1 = Path::new("some_path.txt");
        assert!(!path_1.exists());

        let tempfile = NamedTempFile::new().unwrap();
        let path_2 = tempfile.path();

        // Try to make the below env var unique to avoid race conditions between threads
        let maintenance_log_env = "MAINTENANCE_LOG_PRESENT_3";
        let value = path_2.as_os_str();

        temp_env::with_var(maintenance_log_env, Some(value), || {
            let found = resolve_cfg(path_1, maintenance_log_env).unwrap();
            assert_eq!(found, path_2.to_owned());
        })
    }

    // THIS TEST WILL ATTEMPT TO READ THE ENVIRONMENT
    // ENVIRONMENT VARIABLES ARE PROCESS WIDE ACROSS THREADS
    // IF TWO THREADS ATTEMPT TO SET/UNSET maintenance_log_env
    // THEY WILL INTERFERE WITH EACH OTHER
    // RUN cargo test -- --test-threads=1 TO AVOID RACE CONDITIONS
    // LIKE THE ONE JUST DESCRIBED
    #[test]
    fn test_cfg_resolve_resolve_cfg_path_not_exists_env_var_not_set() {
        let path = Path::new("does_not_exist.txt");
        assert!(!path.exists());
        // Try to make the below env var unique to avoid race conditions between threads
        let maintenance_log_env = "MAINTENANCE_LOG_NOT_PRESENT_3";
        let value: Option<&str> = None;

        temp_env::with_var(maintenance_log_env, value, || {
            let found = resolve_cfg(path, maintenance_log_env);
            assert!(found.is_err());
        });
    }

    #[test]
    fn test_cfg_resolve_resolve_cfg_path_not_exists_skip_env() {
        let path = Path::new("does_not_exist.txt");
        assert!(!path.exists());
        assert!(resolve_cfg(path, "skip").is_err());
    }

    // THIS TEST WILL ATTEMPT TO READ THE ENVIRONMENT
    // ENVIRONMENT VARIABLES ARE PROCESS WIDE ACROSS THREADS
    // IF TWO THREADS ATTEMPT TO SET/UNSET maintenance_log_env
    // THEY WILL INTERFERE WITH EACH OTHER
    // RUN cargo test -- --test-threads=1 TO AVOID RACE CONDITIONS
    // LIKE THE ONE JUST DESCRIBED
    #[test]
    fn test_cfg_resolve_resolve_from_env_var_present() {
        // Try to make the below env var unique to avoid race conditions between threads
        let maintenance_log_env = "MAINTENANCE_LOG_PRESENT_4";
        let value = "path/to/log";
        temp_env::with_var(maintenance_log_env, Some(value), || {
            let found = resolve_from_env(maintenance_log_env);
            assert_eq!(found.unwrap(), PathBuf::from(value));
        })
    }

    // THIS TEST WILL ATTEMPT TO READ THE ENVIRONMENT
    // ENVIRONMENT VARIABLES ARE PROCESS WIDE ACROSS THREADS
    // IF TWO THREADS ATTEMPT TO SET/UNSET maintenance_log_env
    // THEY WILL INTERFERE WITH EACH OTHER
    // RUN cargo test -- --test-threads=1 TO AVOID RACE CONDITIONS
    // LIKE THE ONE JUST DESCRIBED
    #[test]
    fn test_cfg_resolve_resolve_from_env_var_not_present() {
        // Try to make the below env var unique to avoid race conditions between threads
        let maintenance_log_env = "MAINTENANCE_LOG_NOT_PRESENT_4";
        temp_env::with_var(maintenance_log_env, None::<&str>, || {
            assert!(resolve_from_env(maintenance_log_env).is_none())
        })
    }

    #[test]
    fn test_cfg_resolve_resolve_from_env_skip() {
        assert!(resolve_from_env("skip").is_none());
    }
}
