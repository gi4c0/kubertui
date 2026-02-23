use std::{
    process::Stdio,
    thread::sleep,
    time::{Duration, Instant},
};

use anyhow::{Context, anyhow};
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    process::Command,
};

use crate::{
    error::{AppError, AppResult},
    files::{ERROR_FILE_PATH, INFO_FILE_PATH, ensure_app_dir},
};

const TIME_OUT_SECONDS: u64 = 10;

pub async fn start_port_forward(
    namespace: &str,
    pod_name: &str,
    local_port: u16,
    app_port: u16,
) -> AppResult<u32> {
    ensure_app_dir().await?;

    let info_log_file = File::create(INFO_FILE_PATH)
        .await
        .context("Failed to create a port_forward_log file")?;

    let error_log_file = File::create(ERROR_FILE_PATH)
        .await
        .context("Failed to create a port_forward_error file")?;

    let mut child = Command::new("kubectl")
        .kill_on_drop(true)
        .args([
            "port-forward",
            pod_name,
            format!("{}:{}", local_port, app_port).as_str(),
            "-n",
            namespace,
        ])
        .stdin(Stdio::null())
        .stdout(info_log_file.into_std().await)
        .stderr(error_log_file.into_std().await)
        .spawn()
        .context("Failed to start port-forward process")
        .map_err(AppError::PortForwardError)?;

    let mut buf_reader = BufReader::new(
        File::open(INFO_FILE_PATH)
            .await
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
                .await
                .context("failed to open error file path")
                .map_err(AppError::PortForwardError)?
                .read_to_string(&mut logged_error)
                .await
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
            .await
            .context("failed to read from port forward log file")?;

        if line.contains("Forwarding from") {
            break;
        }

        sleep(Duration::from_millis(100));
    }

    let pid = child
        .id()
        .ok_or(anyhow!("no pid for port-forward"))
        .map_err(AppError::PortForwardError)?;

    // Wait for the process to finish and kill the zombie if it was killed outside the program
    tokio::spawn(async move {
        let _ = child.wait().await;
    });

    Ok(pid)
}
