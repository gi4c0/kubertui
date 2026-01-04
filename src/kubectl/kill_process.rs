use std::process::Command;

use anyhow::Context;

use crate::error::{AppError, AppResult};

pub fn kill_process(pid: u32) -> AppResult<()> {
    let output = Command::new("kill")
        .args([pid.to_string()])
        .output()
        .with_context(|| format!("failed to run a kill process: {pid}"))
        .map_err(AppError::KillProcess)?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);

        return Err(AppError::KillProcess(anyhow::anyhow!(
            "kill process failed: {error}"
        )));
    }

    Ok(())
}
