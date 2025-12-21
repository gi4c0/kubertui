use anyhow::Context;
use serde::Deserialize;
use tokio::process::Command;

use crate::error::AppError;

#[derive(Deserialize)]
struct Response {
    items: Vec<Namespace>,
}

#[derive(Deserialize)]
struct Namespace {
    metadata: Metadata,
}

#[derive(Deserialize)]
struct Metadata {
    name: String,
}

pub async fn get_namespaces() -> Result<Vec<String>, AppError> {
    let output = Command::new("kubectl")
        .args(["get", "namespaces", "-o", "json"])
        .output()
        .await
        .context("Failed to run command 'kubectl get namespaces'")
        .map_err(AppError::FailLoadNamespaces)?;

    if !output.status.success() {
        return Err(AppError::FailLoadNamespaces(anyhow::anyhow!(
            "Got error from command: 'kubectl get namespaces'\nstderr: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let parsed: Response = serde_json::from_slice(&output.stdout)
        .context("Invalid JSON")
        .map_err(AppError::FailLoadNamespaces)?;

    Ok(parsed
        .items
        .into_iter()
        .map(|item| item.metadata.name)
        .collect())
}
