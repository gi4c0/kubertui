use std::process::Command;

use anyhow::Context;
use serde::Deserialize;

use crate::error::{AppError, AppResult};

mod kill_process;
pub mod namespace;
mod pod_logs;
pub mod pods;
mod port_forward;

pub use kill_process::*;
pub use pod_logs::*;
pub use port_forward::*;

fn run_kubectl_command<T: for<'a> Deserialize<'a>>(command: &str, args: Vec<&str>) -> AppResult<T> {
    let output = Command::new(command)
        .args(&args)
        .output()
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
