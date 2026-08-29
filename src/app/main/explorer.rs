use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect};
use serde::{Deserialize, Serialize};

use crate::{
    app::{
        cache::ExplorerCache,
        events::{EventSender, KeyEventResult},
        main::explorer::{
            clusters_list::ClustersList, namespaces_list::NamespacesList, pods::Pods,
        },
    },
    error::AppResult,
    kubectl::pods::Pod,
};

pub mod clusters_list;
pub mod namespaces_list;
pub mod pods;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum ExplorerKind {
    Clusters,
    Namespaces,
    Pods,
}

#[derive(Debug, Clone)]
pub struct Explorer {
    kind: ExplorerKind,
    clusters: ClustersList,
    namespaces: NamespacesList,
    pods: Pods,
}

impl Explorer {
    pub fn new(event_sender: EventSender) -> Self {
        Self {
            kind: ExplorerKind::Clusters,
            clusters: ClustersList::new(event_sender.clone()),
            namespaces: NamespacesList::new(event_sender.clone()),
            pods: Pods::new(event_sender.clone()),
        }
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame) {
        match self.kind {
            ExplorerKind::Clusters => self.clusters.draw(area, frame),
            ExplorerKind::Pods => self.pods.draw(area, frame),
            ExplorerKind::Namespaces => self.namespaces.draw(area, frame),
        }
    }

    pub fn stop_pods_spinner(&mut self, pod_name: &str) {
        self.pods.stop_spinner(pod_name);
    }

    pub fn set_kind(&mut self, kind: ExplorerKind) {
        self.kind = kind;
    }

    pub fn show_namespaces(&mut self, namespaces: Vec<String>) {
        self.kind = ExplorerKind::Namespaces;
        self.namespaces.set_namespaces(namespaces);
    }

    pub fn set_clusters(&mut self, clusters: Vec<String>) {
        self.clusters.set_clusters(clusters);
    }

    pub fn pods_updated(&mut self, namespace: &str, pods: Vec<Pod>) {
        self.pods.pods_updated(namespace, pods);
    }

    pub fn show_pods(&mut self, namespace: String, pods: Vec<Pod>) {
        self.kind = ExplorerKind::Pods;
        self.pods.show_pods(namespace, pods);
    }

    pub async fn initial_load(&mut self) -> AppResult<()> {
        self.clusters.load_clusters().await
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> KeyEventResult {
        match self.kind {
            ExplorerKind::Clusters => self.clusters.handle_key_event(key),
            ExplorerKind::Namespaces => self.namespaces.handle_key_event(key),
            ExplorerKind::Pods => self.pods.handle_key_event(key),
        }
    }

    pub fn from_cache(value: ExplorerCache, event_sender: EventSender) -> Self {
        Self {
            pods: Pods::from_cache(value.pods, event_sender.clone()),
            kind: value.kind,
            clusters: ClustersList::from_cache(value.clusters, event_sender.clone()),
            namespaces: NamespacesList::from_cache(value.namespaces, event_sender.clone()),
        }
    }
}

impl From<Explorer> for ExplorerCache {
    fn from(value: Explorer) -> Self {
        Self {
            kind: value.kind,
            clusters: value.clusters.into(),
            namespaces: value.namespaces.into(),
            pods: value.pods.into(),
        }
    }
}
