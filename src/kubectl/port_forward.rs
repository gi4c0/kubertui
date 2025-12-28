use anyhow::Context;
use tokio::process::Command;

use crate::error::{AppError, AppResult};

pub async fn start_port_forward(
    namespace: &str,
    pod_name: &str,
    local_port: u16,
    app_port: u16,
) -> AppResult<()> {
    let output = Command::new("kubectl")
        .args([
            "port-forward",
            pod_name,
            format!("{}:{}", local_port, app_port).as_str(),
            "-n",
            namespace,
        ])
        .output()
        .await
        .with_context(|| format!("Failed to run port-forward command {local_port}:{app_port}"))
        .map_err(AppError::FailedRunKubeCtlCommand)?;

    // TODO: handle taken port error
    if !output.status.success() {
        return Err(AppError::FailedRunKubeCtlCommand(anyhow::anyhow!(
            "Failed to run port-forward command {local_port}:{app_port}"
        )));
    }

    Ok(())
}
