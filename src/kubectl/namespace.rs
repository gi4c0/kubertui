use serde::Deserialize;

use crate::{error::AppError, kubectl::run_kubectl_command};

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
    let parsed: Response =
        run_kubectl_command("kubectl", vec!["get", "namespaces", "-o", "json"]).await?;

    Ok(parsed
        .items
        .into_iter()
        .map(|item| item.metadata.name)
        .collect())
}
