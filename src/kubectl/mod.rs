use anyhow::Context;
use serde::Deserialize;
use tokio::process::Command;

use crate::error::{AppError, AppResult};

pub mod namespace;
pub mod pods;
mod port_forward;

pub use port_forward::*;

async fn run_kubectl_command<T: for<'a> Deserialize<'a>>(
    command: &str,
    args: Vec<&str>,
) -> AppResult<T> {
    let output = Command::new(command)
        .args(&args)
        .output()
        .await
        .with_context(|| format!("Failed to run command {} '{}'", command, args.join(" ")))
        .map_err(AppError::FailedRunKubeCtlCommand)?;

    if !output.status.success() {
        return Err(AppError::FailedRunKubeCtlCommand(anyhow::anyhow!(
            "Got error from command: {} '{}'\nstderr: {}",
            command,
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let parsed: T = serde_json::from_slice(&output.stdout)
        .context("Invalid JSON")
        .map_err(AppError::FailedRunKubeCtlCommand)?;

    Ok(parsed)
}
