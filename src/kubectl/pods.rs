use serde::{Deserialize, Serialize};

use crate::{error::AppResult, kubectl::run_kubectl_command};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pod {
    pub name: String,
    pub container_statuses: Vec<PodStatus>,
    pub containers: Vec<PodContainer>,
    pub reason: Option<String>,
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
            reason: item.status.reason,
            container_statuses: item
                .status
                .container_statuses
                .unwrap_or(vec![])
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
                        .unwrap_or(vec![])
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
    ports: Option<Vec<ContainerPort>>,
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
    container_statuses: Option<Vec<ContainerStatus>>,
    reason: Option<String>,
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
        #[serde(rename = "containerID")]
        container_id: Option<String>,
        exit_code: usize,
        finished_at: Option<String>,
        reason: String,
        message: Option<String>,
        started_at: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Waiting {
        reason: String,
        message: Option<String>,
    },

    #[serde(rename_all = "camelCase")]
    Running { started_at: String },
}
