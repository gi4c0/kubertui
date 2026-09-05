use crate::app::common::handle_general_keys;
use crate::app::events::PodMenuEvent;
use crate::app::main_window::explorer::Explorer;
use crate::app::main_window::port_forwards::PortForwardsList;
use crate::app::{MainWindowKind, main_window::logs::Logs};
use crate::error::AppResult;
use ratatui::{Frame, crossterm::event::KeyEvent, layout::Rect};

use crate::app::{
    cache::MainWindowCache,
    events::{EventSender, ExplorerEvent, KeyEventResult, LogsEvent},
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

    pub fn handle_explorer_event(&mut self, event: ExplorerEvent) {
        self.explorer.handle_event(event);
    }

    pub fn handle_logs_event(&mut self, event: LogsEvent) {
        self.logs.handle_event(event);
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

    pub fn set_kind(&mut self, kind: MainWindowKind) {
        self.kind = kind;
    }
}
