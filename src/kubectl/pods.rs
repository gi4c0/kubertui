use serde::{Deserialize, Serialize};

use crate::{error::AppResult, kubectl::run_kubectl_command};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pod {
    pub name: String,
    pub container_statuses: Vec<PodStatus>,
    pub containers: Vec<PodContainer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodContainer {
    pub name: String,
    // TODO: There might be multiple ports.
    pub port: u16,
}

impl AsRef<str> for PodContainer {
    fn as_ref(&self) -> &str {
        self.name.as_str()
    }
}

pub async fn get_pods_list(namespace: &str) -> AppResult<Vec<Pod>> {
    let parsed: ApiResponse = run_kubectl_command(
        "kubectl",
        vec!["get", "pods", "-n", namespace, "-o", "json"],
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
            containers: item
                .spec
                .containers
                .into_iter()
                .map(|item| PodContainer {
                    name: item.name,
                    port: item
                        .ports
                        .first()
                        .map(|port| port.container_port)
                        .unwrap_or(0),
                })
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
    spec: Spec,
}

#[derive(Deserialize)]
struct Spec {
    containers: Vec<Container>,
}

#[derive(Deserialize)]
struct Container {
    name: String,
    ports: Vec<ContainerPort>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContainerPort {
    container_port: u16,
    // name: String,
    // protocol: String,
}

#[derive(Deserialize)]
struct Metadata {
    name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Status {
    container_statuses: Vec<ContainerStatus>,
}

#[derive(Deserialize)]
struct ContainerStatus {
    state: PodStatus,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum PodStatus {
    Known(KnownPodStatus),
    Unknown(serde_json::Value),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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
