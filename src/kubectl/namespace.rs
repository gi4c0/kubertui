use serde::Deserialize;

use crate::{
    error::{AppError, AppResult},
    kubectl::{run_kubectl_command, run_kubectl_command_and_parse},
};

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

pub async fn get_namespaces(cluster: &str) -> Result<Vec<String>, AppError> {
    switch_context(cluster).await?;

    let parsed: Response = run_kubectl_command_and_parse(
        "kubectl",
        vec!["get", "namespaces", "--cluster", cluster, "-o", "json"],
    )
    .await?;

    Ok(parsed
        .items
        .into_iter()
        .map(|item| item.metadata.name)
        .collect())
}

async fn switch_context(cluster: &str) -> AppResult<()> {
    let output = run_kubectl_command("kubectl", &["config", "use-context", cluster]).await?;
    let output = String::from_utf8_lossy(&output);

    if !output.contains(&format!("Switched to context \"{cluster}\"")) {
        return Err(AppError::FailedRunKubeCtlCommand(anyhow::anyhow!(
            "Failed to switch context: {output}",
        )));
    }

    Ok(())
}
