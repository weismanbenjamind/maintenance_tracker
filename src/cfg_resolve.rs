use crate::errors::CfgResolveError;
use log::info;
use std::path::{Path, PathBuf};

const SKIP_ENV_FLAG: &str = "skip";

pub fn resolve_cfg(
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

fn resolve_from_env(maintenance_log_env: &str) -> Option<PathBuf> {
    match maintenance_log_env == SKIP_ENV_FLAG {
        true => None,
        false => std::env::var(maintenance_log_env).ok().map(PathBuf::from),
    }
}
