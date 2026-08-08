use anyhow::Context;
use serde::Deserialize;
use tokio::process::Command;

use crate::error::{AppError, AppResult};

mod delete_pod;
mod get_clusters;
mod kill_process;
mod namespace;
mod pod_logs;
pub mod pods;
mod port_forward;

pub use delete_pod::*;
pub use get_clusters::*;
pub use kill_process::*;
pub use namespace::*;
pub use pod_logs::*;
pub use port_forward::*;

async fn run_kubectl_command(command: &str, args: &[&str]) -> AppResult<Vec<u8>> {
    let output = Command::new(command)
        .args(args)
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

    Ok(output.stdout)
}

async fn run_kubectl_command_and_parse<T: for<'a> Deserialize<'a>>(
    command: &str,
    args: Vec<&str>,
) -> AppResult<T> {
    let output = run_kubectl_command(command, &args).await?;

    let parsed: T = serde_json::from_slice(&output)
        .with_context(|| {
            format!(
                "invalid JSON from command: '{} {}'",
                command,
                args.join(" ")
            )
        })
        .map_err(AppError::FailedRunKubeCtlCommand)?;

    Ok(parsed)
}
