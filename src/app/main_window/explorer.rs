use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect};
use serde::{Deserialize, Serialize};

use crate::{
    app::{
        cache::ExplorerCache,
        events::{EventSender, ExplorerEvent, KeyEventResult},
        main_window::explorer::{
            clusters_list::ClustersList, namespaces_list::NamespacesList, pods_pane::PodsPane,
        },
    },
    error::AppResult,
};

pub mod clusters_list;
pub mod namespaces_list;
pub mod pods_pane;

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
    pods: PodsPane,
}

impl Explorer {
    pub fn new(event_sender: EventSender) -> Self {
        Self {
            kind: ExplorerKind::Clusters,
            clusters: ClustersList::new(event_sender.clone()),
            namespaces: NamespacesList::new(event_sender.clone()),
            pods: PodsPane::new(event_sender.clone()),
        }
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame) {
        match self.kind {
            ExplorerKind::Clusters => self.clusters.draw(area, frame),
            ExplorerKind::Pods => self.pods.draw(area, frame),
            ExplorerKind::Namespaces => self.namespaces.draw(area, frame),
        }
    }

    pub fn handle_event(&mut self, event: ExplorerEvent) {
        match event {
            ExplorerEvent::Show(kind) => self.kind = kind,

            ExplorerEvent::ClustersLoaded(clusters) => self.clusters.set_clusters(clusters),

            ExplorerEvent::NamespacesLoaded { namespaces, .. } => {
                self.kind = ExplorerKind::Namespaces;
                self.namespaces.set_namespaces(namespaces);
            }

            ExplorerEvent::PodsLoaded { namespace, pods } => {
                self.kind = ExplorerKind::Pods;
                self.pods.show_pods(namespace, pods);
            }

            ExplorerEvent::PodsUpdated { namespace, pods } => {
                self.pods.pods_updated(&namespace, pods);
            }

            ExplorerEvent::PodLogsFinished { pod_name } => self.pods.stop_spinner(&pod_name),
        }
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
            pods: PodsPane::from_cache(value.pods, event_sender.clone()),
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
