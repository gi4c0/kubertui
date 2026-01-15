use anyhow::Context;
use tokio::fs;

use crate::error::{AppError, AppResult};

pub const DIR_PATH: &str = "/tmp/kubertui";
pub const CACHE_PATH: &str = "/tmp/kubertui/cache.json";
pub const ERROR_FILE_PATH: &str = "/tmp/kubertui/port_forward_error.log";
pub const INFO_FILE_PATH: &str = "/tmp/kubertui/port_forward_info.log";

pub async fn ensure_app_dir() -> AppResult<()> {
    fs::create_dir_all(DIR_PATH)
        .await
        .with_context(|| format!("failed to create cache dir: {DIR_PATH}"))
        .map_err(AppError::CacheError)?;

    Ok(())
}
