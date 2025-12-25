use serde::Deserialize;

use crate::{error::AppResult, kubectl::run_kubectl_command};

#[derive(Clone)]
pub struct Pod {
    pub name: String,
    pub container_statuses: Vec<PodStatus>,
}

pub async fn get_pods_list(namespace: String) -> AppResult<Vec<Pod>> {
    let parsed: ApiResponse = run_kubectl_command(
        "kubectl",
        vec!["get", "pods", "-n", namespace.as_str(), "-o", "json"],
    )
    .await?;

    Ok(parsed
        .items
        .into_iter()
        .map(|item| Pod {
            name: item.metadata.name,
            container_statuses: item
                .status
                .container_statuses
                .into_iter()
                .map(|item| item.state)
                .collect(),
        })
        .collect())
}

#[derive(Deserialize)]
struct ApiResponse {
    items: Vec<Item>,
}

#[derive(Deserialize)]
struct Item {
    metadata: Metadata,
    status: Status,
}

#[derive(Deserialize)]
struct Metadata {
    name: String,
}

#[derive(Deserialize)]
struct Status {
    #[serde(rename = "containerStatuses")]
    container_statuses: Vec<ContainerStatus>,
}

#[derive(Deserialize)]
struct ContainerStatus {
    state: PodStatus,
}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum PodStatus {
    Known(KnownPodStatus),
    Unknown(serde_json::Value),
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum KnownPodStatus {
    #[serde(rename_all = "camelCase")]
    Terminated {
        container_id: String,
        exit_code: usize,
        finished_at: String,
        reason: String,
        started_at: String,
    },
    #[serde(rename_all = "camelCase")]
    Waiting {
        reason: String,
        message: Option<String>,
    },

    #[serde(rename_all = "camelCase")]
    Running { started_at: String },
}
