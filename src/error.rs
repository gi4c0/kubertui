use std::io;

use anyhow::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("failed to load namespaces")]
    FailedRunKubeCtlCommand(Error),

    #[error("error from terminal")]
    TerminalError(#[from] io::Error),
}

pub type AppResult<T> = Result<T, AppError>;
