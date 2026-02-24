use anyhow::Context;
use tokio::process::Command;

use crate::error::{AppError, AppResult};

pub async fn load_logs(pod_name: &str) -> AppResult<Vec<String>> {
    let output = Command::new("kubectl")
        .args(["logs", pod_name])
        .output()
        .await
        .context("Failed to run logs command")
        .map_err(AppError::FailedRunKubeCtlCommand)?;

    if !output.status.success() {
        return Err(AppError::FailedRunKubeCtlCommand(anyhow::anyhow!(
            "Got error from logs command.\nstderr: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let result: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.to_owned())
        .collect::<Vec<_>>();

    Ok(result)
}
