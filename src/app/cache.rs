use anyhow::Context;
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::{
    app::{ActiveWindow, App, MainWindow, side_bar::port_forwards::PortForward},
    error::{AppError, AppResult},
    kubectl::pods::{Pod, PodContainer},
};

const FILE_PATH: &str = "/tmp/kubertui/cache.json";

pub async fn save_cache(app: &App) -> AppResult<()> {
    let cache_payload = AppCache {
        namespaces: app.namespaces.clone().into(),
        pods: match &app.pods {
            Some(pods) => Some(pods.clone().into()),
            None => None,
        },
        exit: app.exit,
        active_window: app.active_window,
        main_window: app.main_window,
        side_bar: app.side_bar.clone().into(),
    };

    let json = serde_json::to_string(&cache_payload)
        .context("failed to serialize cache")
        .map_err(AppError::CacheError)?;

    fs::write(FILE_PATH, json)
        .await
        .context("failed to write json cache to file")
        .map_err(AppError::CacheError)?;

    Ok(())
}

pub async fn read_cache() -> AppResult<AppCache> {
    let content = fs::read(FILE_PATH)
        .await
        .context("failed to read cache into string")
        .map_err(AppError::CacheError)?;

    let cache: AppCache =
        serde_json::from_slice(&content).context("failed to deserialize cache")?;

    Ok(cache)
}
#[derive(Debug, Serialize, Deserialize)]
pub struct AppCache {
    pub namespaces: NamespacesListCache,
    pub pods: Option<PodsListCache>,
    pub side_bar: SideBarCache,
    pub exit: bool,
    pub main_window: MainWindow,
    pub active_window: ActiveWindow,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SideBarCache {
    pub recent_namespaces: RecentNamespacesListCache,
    pub port_forwards: PortForwardsListCache,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PortForwardsListCache {
    pub list: Vec<PortForward>,
    pub state: StateCache,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecentNamespacesListCache {
    pub state: StateCache,
    pub list: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PodsListCache {
    pub original_list: Vec<Pod>,
    pub filtered_list: Vec<Pod>,
    pub state: StateCache,
    pub filter: String,
    pub is_filter_mod: bool,
    pub longest_name: u16,
    pub port_forward_popup: Option<PortForwardPopupCache>,
    pub namespace: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PortForwardPopupCache {
    pub port: String,
    pub pod_containers: Vec<PodContainer>,
    pub state: StateCache,
    pub selected_container: Option<PodContainer>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NamespacesListCache {
    pub original_list: Vec<String>,
    pub filtered_list: Vec<String>,
    pub filter: String,
    pub is_filter_mod: bool,
    pub state: StateCache,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StateCache {
    pub selected: Option<usize>,
}
