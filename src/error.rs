use std::io;

use anyhow::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("failed to forward ports: {0}")]
    PortForwardError(Error),

    #[error("failed to save/retrieve cache: {0}")]
    CacheError(Error),

    #[error("failed to load namespaces: {0}")]
    FailedRunKubeCtlCommand(Error),

    #[error("error from terminal: {0}")]
    TerminalError(#[from] io::Error),

    #[error(transparent)]
    GeneralError(#[from] Error),
}

pub type AppResult<T> = Result<T, AppError>;
