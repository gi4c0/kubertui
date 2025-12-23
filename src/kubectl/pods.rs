use serde::Deserialize;

use crate::{error::AppResult, kubectl::run_kubectl_command};

pub struct Pod {
    pub name: String,
    pub status: Vec<PodStatus>,
}

pub async fn get_pods_list(namespace: &str) -> AppResult<Vec<Pod>> {
    let parsed: ApiResponse =
        run_kubectl_command("kubectl", vec!["get", "namespaces", "-o", "json"]).await?;

    Ok(parsed
        .items
        .into_iter()
        .map(|item| Pod {
            name: item.metadata.name,
            status: item
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

#[derive(Deserialize)]
#[serde(untagged)]
pub enum PodStatus {
    Known(KnownPodStatus),
    Unknown(serde_json::Value),
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KnownPodStatus {
    #[serde(rename_all = "camelCase")]
    Terminated {
        container_id: String,
        exit_code: usize,
        finished_at: String,
        reason: String,
        started_at: String,
    },
    Waiting {
        reason: String,
        message: Option<String>,
    },

    #[serde(rename_all = "camelCase")]
    Running { started_at: String },
}
