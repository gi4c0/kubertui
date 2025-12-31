use std::process::Stdio;

use anyhow::anyhow;
use tokio::process::Command;

use crate::error::{AppError, AppResult};

pub async fn start_port_forward(
    namespace: &str,
    pod_name: &str,
    local_port: u16,
    app_port: u16,
) -> AppResult<u32> {
    // let output = Command::new("kubectl")
    //     .args([
    //         "port-forward",
    //         pod_name,
    //         format!("{}:{}", local_port, app_port).as_str(),
    //         "-n",
    //         namespace,
    //     ])
    //     .output()
    //     .await
    //     .with_context(|| format!("Failed to run port-forward command {local_port}:{app_port}"))
    //     .map_err(AppError::FailedRunKubeCtlCommand)?;
    //
    // // TODO: handle taken port error
    // if !output.status.success() {
    //     return Err(AppError::FailedRunKubeCtlCommand(anyhow::anyhow!(
    //         "Failed to run port-forward command {local_port}:{app_port}"
    //     )));
    // }
    //
    // Ok(())

    let output = unsafe {
        Command::new("kubectl")
            .args([
                "port-forward",
                pod_name,
                format!("{}:{}", local_port, app_port).as_str(),
                "-n",
                namespace,
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            // TODO: log errors
            .stderr(Stdio::null())
            .pre_exec(|| {
                // Try to create a new session
                // This call is unsafe because it runs in the child process
                // after fork but before exec.
                let pid = libc::setsid();
                if pid == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            })
            .spawn()
    };

    match output {
        Err(e) => panic!("Failed to start port-forward process{e}"),
        Ok(process) => process.id().ok_or(AppError::PortForwardError(anyhow!(
            "No PID from port-forward"
        ))),
    }
}
