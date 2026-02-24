use anyhow::Context;
use tokio::process::Command;

use crate::error::{AppError, AppResult};

pub async fn delete_pod(namespace: &str, pod_name: &str) -> AppResult<()> {
    let command = "kubectl";
    let args = ["delete", "pod", "--namespace", namespace, pod_name];

    let output = Command::new(command)
        .args(args)
        .output()
        .await
        .with_context(|| format!("Failed to run command '{} {}'", command, args.join(" ")))
        .map_err(AppError::FailedRunKubeCtlCommand)?;

    if !output.status.success() {
        return Err(AppError::FailedRunKubeCtlCommand(anyhow::anyhow!(
            "Got error from command: {} '{}'\nstderr: {}",
            command,
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let expected_response = format!("pod \"{pod_name}\" deleted\n");
    let response = String::from_utf8_lossy(&output.stdout);

    if response.as_ref() != expected_response.as_str() {
        return Err(AppError::FailedRunKubeCtlCommand(anyhow::anyhow!(
            "Failed to delete pod {pod_name}:\n{response}"
        )));
    }

    Ok(())
}
