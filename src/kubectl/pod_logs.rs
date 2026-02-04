use anyhow::Context;
use serde::{Deserialize, Serialize};
use strum::Display;
use tokio::process::Command;

use crate::error::{AppError, AppResult};

// #[derive(Debug, Clone, Deserialize, Serialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Log {
//     pub level: LogLevel,
//     pub timestamp: u64,
//     pub time: String,
//     pub msg: String,
//     pub request_id: Option<String>,
//     pub action: Option<String>,
//     pub context: Option<String>,
//     pub time_spend: Option<u64>,
// }
//
// #[derive(Debug, Clone, Copy, Deserialize, Serialize, Display)]
// pub enum LogLevel {
//     Info,
//     Warn,
//     Error,
// }

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
