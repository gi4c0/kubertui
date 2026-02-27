use std::io::ErrorKind;

use anyhow::Context;
use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::{
    app::{ActiveWindow, App, MainWindowKind, side_bar::rootspaces::RootSpaceWindowKind},
    error::{AppError, AppResult},
    files::{CACHE_PATH, ensure_app_dir},
    kubectl::pods::{Pod, PodContainer},
};

pub async fn save_cache(app: &App) -> AppResult<()> {
    ensure_app_dir().await?;

    let cache_payload = AppCache {
        main: app.main.clone().into(),
        active_window: app.active_window,
        side_bar: app.side_bar.clone().into(),
    };

    let json = serde_json::to_string(&cache_payload)
        .context("failed to serialize cache")
        .map_err(AppError::CacheError)?;

    fs::write(CACHE_PATH, json)
        .await
        .context("failed to write json cache to file")
        .map_err(AppError::CacheError)?;

    Ok(())
}

pub async fn read_cache() -> Option<AppCache> {
    let content = match fs::read(CACHE_PATH).await {
        Ok(content) => content,
        Err(err) => {
            if err.kind() == ErrorKind::NotFound {
                return None;
            }

            // TODO: handle error: show warning or something
            todo!()
            // return Err(AppError::CacheError(anyhow::format_err!(
            //     "failed to read cache into string: {:?}",
            //     err
            // )));
        }
    };

    let cache: Option<AppCache> = serde_json::from_slice(&content).ok();
    cache
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MainWindowCache {
    pub pods_list: Option<PodsListCache>,
    pub kind: MainWindowKind,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppCache {
    pub main: MainWindowCache,
    pub side_bar: SideBarCache,
    pub active_window: ActiveWindow,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RootSpaceCache {
    pub recent: RecentNamespacesListCache,
    pub full_list: NamespacesListCache,
    pub kind: RootSpaceWindowKind,
    pub clusters: ClustersListCache,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClustersListCache {
    pub list: FilterableListCache<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SideBarCache {
    pub namespaces: RootSpaceCache,
    pub port_forwards: PortForwardsListCache,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PortForwardsListCache {
    pub list: FilterableListCache<PortForwardCache>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PortForwardCache {
    pub namespace: String,
    pub pod_name: String,
    pub local_port: u16,
    pub app_port: u16,
    pub pid: Option<u32>,
    pub item_str: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecentNamespacesListCache {
    pub recent_namespaces_list: FilterableListCache<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PodsListCache {
    pub original_list: Vec<Pod>,
    pub filtered_list: Vec<usize>,
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
    pub pod_containers: FilterableListCache<PodContainer>,
    pub selected_container: Option<PodContainer>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NamespacesListCache {
    pub cluster: String,
    pub namespace_list: FilterableListCache<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StateCache {
    pub selected: Option<usize>,
}

impl From<ListState> for StateCache {
    fn from(value: ListState) -> Self {
        Self {
            selected: value.selected(),
        }
    }
}

impl From<StateCache> for ListState {
    fn from(value: StateCache) -> Self {
        let mut state = ListState::default();
        state.select(value.selected);
        state
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilterableListCache<T> {
    pub list: Vec<T>,
    pub state: StateCache,
    pub title: String,
    pub is_filterable: bool,
    pub filtered_list: Vec<usize>,
    pub filter: String,
    pub is_filter_mod: bool,
}
