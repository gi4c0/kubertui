use crate::app::common::handle_general_keys;
use crate::app::main_window::explorer::{Explorer, ExplorerKind};
use crate::app::main_window::port_forwards::PortForwardsList;
use crate::app::{MainWindowKind, main_window::logs::Logs};
use crate::error::AppResult;
use ratatui::{Frame, crossterm::event::KeyEvent, layout::Rect};

use crate::{
    app::{
        cache::MainWindowCache,
        events::{EventSender, KeyEventResult},
    },
    kubectl::pods::Pod,
};

pub mod explorer;
pub mod logs;
mod port_forwards;

#[derive(Debug, Clone)]
pub struct MainWindow {
    event_sender: EventSender,
    kind: MainWindowKind,
    explorer: Explorer,
    port_forwards: PortForwardsList,
    logs: Logs,
}

pub struct NamespacePod {
    pub namespace: String,
    pub pod: String,
}

impl From<MainWindow> for MainWindowCache {
    fn from(value: MainWindow) -> Self {
        Self {
            kind: value.kind,
            explorer: value.explorer.into(),
            port_forwards: value.port_forwards.into(),
        }
    }
}

impl MainWindow {
    pub fn new(event_sender: EventSender) -> Self {
        Self {
            event_sender: event_sender.clone(),
            kind: MainWindowKind::Explorer,
            explorer: Explorer::new(event_sender.clone()),
            port_forwards: PortForwardsList::new(event_sender.clone()),
            logs: Logs::new(event_sender.clone()),
        }
    }

    pub fn show_namespaces(&mut self, namespaces: Vec<String>) {
        self.explorer.show_namespaces(namespaces);
    }

    pub fn add_to_list_and_port_forward(
        &mut self,
        namespace: String,
        pod_name: String,
        local_port: u16,
        app_port: u16,
    ) {
        self.port_forwards
            .add_to_list_and_port_forward(namespace, pod_name, local_port, app_port);
    }

    pub fn set_clusters(&mut self, clusters: Vec<String>) {
        self.explorer.set_clusters(clusters);
    }

    pub fn reload_logs(&self) {
        self.logs.reload_logs();
    }

    pub fn stop_pods_spinner(&mut self, pod_name: &str) {
        self.explorer.stop_pods_spinner(pod_name);
    }

    pub fn pods_updated(&mut self, namespace: &str, pods: Vec<Pod>) {
        self.explorer.pods_updated(namespace, pods);
    }

    pub fn logs_reloaded(&mut self, pod_name: String, logs: Vec<String>) {
        self.logs.logs_reloaded(pod_name, logs);
    }

    pub fn load_logs(&self, namespace: String, pod_name: String) {
        Logs::load_logs(namespace, pod_name, self.event_sender.clone());
    }

    pub fn set_logs(&mut self, namespace: String, pod_name: String, logs: Vec<String>) {
        self.logs.add_pod_logs(namespace, pod_name, logs);
    }

    pub fn show_pods(&mut self, namespace: String, pods: Vec<Pod>) {
        self.explorer.show_pods(namespace, pods);
    }

    pub async fn initial_load(&mut self) -> AppResult<()> {
        self.explorer.initial_load().await
    }

    pub fn from_cache(value: MainWindowCache, event_sender: EventSender) -> Self {
        Self {
            kind: value.kind,
            explorer: Explorer::from_cache(value.explorer, event_sender.clone()),
            port_forwards: PortForwardsList::from_cache(value.port_forwards, event_sender.clone()),
            event_sender: event_sender.clone(),
            logs: Logs::new(event_sender.clone()),
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        let result = match self.kind {
            MainWindowKind::Explorer => self.explorer.handle_key_event(key),
            MainWindowKind::PortForward => self.port_forwards.handle_key_event(key),
            MainWindowKind::Logs => self.logs.handle_key_event(key),
        };

        if result == KeyEventResult::Ignored {
            handle_general_keys(key, &self.event_sender);
        }
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame) {
        match self.kind {
            MainWindowKind::Explorer => self.explorer.draw(area, frame),
            MainWindowKind::PortForward => self.port_forwards.draw(area, frame),
            MainWindowKind::Logs => self.logs.draw(area, frame),
        };
    }

    pub fn set_active_window(
        &mut self,
        new_active_window: MainWindowKind,
        explorer_kind: Option<ExplorerKind>,
    ) {
        self.kind = new_active_window;
        if let Some(kind) = explorer_kind {
            self.explorer.set_kind(kind);
        }
    }
}
