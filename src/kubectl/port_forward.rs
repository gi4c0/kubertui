use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
    os::unix::process::CommandExt,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

use anyhow::Context;
use tokio::time::sleep;

use crate::{
    error::{AppError, AppResult},
    files::{ERROR_FILE_PATH, INFO_FILE_PATH, ensure_app_dir},
};

const TIME_OUT_SECONDS: u64 = 3;

pub async fn start_port_forward(
    namespace: &str,
    pod_name: &str,
    local_port: u16,
    app_port: u16,
) -> AppResult<u32> {
    ensure_app_dir().await?;

    let info_log_file =
        File::create(INFO_FILE_PATH).context("Failed to create a port_forward_log file")?;

    let error_log_file =
        File::create(ERROR_FILE_PATH).context("Failed to create a port_forward_error file")?;

    let pid = unsafe {
        Command::new("kubectl")
            .args([
                "port-forward",
                pod_name,
                format!("{}:{}", local_port, app_port).as_str(),
                "-n",
                namespace,
            ])
            .stdin(Stdio::null())
            .stdout(info_log_file)
            .stderr(error_log_file)
            .pre_exec(|| {
                // Try to create a new session
                // This call is unsafe because it runs in the child process after fork but before exec.
                let pid = libc::setsid();
                if pid == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            })
            .spawn()
            .context("Failed to start port-forward process")
            .map_err(AppError::PortForwardError)?
            .id()
    };

    let mut buf_reader = BufReader::new(
        File::open(INFO_FILE_PATH)
            .context("failed to open info file path")
            .map_err(AppError::PortForwardError)?,
    );

    let mut line = String::new();

    let now = Instant::now();
    let timeout = Duration::from_secs(TIME_OUT_SECONDS);

    loop {
        if now.elapsed() > timeout {
            let mut logged_error = String::new();

            File::open(ERROR_FILE_PATH)
                .context("failed to open error file path")
                .map_err(AppError::PortForwardError)?
                .read_to_string(&mut logged_error)
                .context("failed to read error from log file")
                .map_err(AppError::PortForwardError)?;

            let error_message = if !logged_error.is_empty() {
                logged_error
            } else {
                format!("Port Forward timed out after {TIME_OUT_SECONDS}")
            };

            return Err(AppError::PortForwardError(anyhow::anyhow!(error_message)));
        }

        line.clear();

        buf_reader
            .read_line(&mut line)
            .context("failed to read from port forward log file")?;

        if line.contains("Forwarding from") {
            break;
        }

        sleep(Duration::from_millis(100)).await;
    }

    Ok(pid)
}

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
