use ratatui::{
    Frame,
    layout::{Constraint, Direction, Flex, HorizontalAlignment, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Clear,
};
use serde::{Deserialize, Serialize};
use strum::VariantArray;

use crate::app::{MainWindowKind, common::build_block, main_window::explorer::ExplorerKind};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct HeaderExplorerData {
    cluster: Option<String>,
    namespace: Option<String>,
}

impl HeaderExplorerData {
    fn breadcrumb(&self) -> String {
        match (&self.cluster, &self.namespace) {
            (Some(cluster), Some(namespace)) => format!("{cluster} -> {namespace} -> Pods"),
            (Some(cluster), None) => format!("{cluster} -> Namespaces"),
            _ => "Clusters".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Header {
    active: MainWindowKind,
    explorer_kind: ExplorerKind,
    explorer_data: HeaderExplorerData,
}

impl Header {
    pub fn new() -> Self {
        Self {
            active: MainWindowKind::Explorer,
            explorer_kind: ExplorerKind::Clusters,
            explorer_data: HeaderExplorerData::default(),
        }
    }

    pub fn set_active(&mut self, new_active: MainWindowKind) {
        self.active = new_active;
    }

    pub fn set_cluster(&mut self, cluster: String) {
        self.explorer_data.cluster = Some(cluster);
    }

    pub fn set_namespace(&mut self, namespace: String) {
        self.explorer_data.namespace = Some(namespace);
    }

    pub fn set_explorer_kind(&mut self, explorer_kind: ExplorerKind) {
        self.explorer_kind = explorer_kind;

        match self.explorer_kind {
            ExplorerKind::Clusters => self.explorer_data = HeaderExplorerData::default(),
            ExplorerKind::Namespaces => self.explorer_data.namespace = None,
            _ => {}
        };
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame) {
        let tabs = MainWindowKind::VARIANTS;

        let [centered_area] = Layout::vertical([Constraint::Length(1)])
            .flex(Flex::Center)
            .areas(area);

        let layouts = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(tabs.iter().map(|_| Constraint::Fill(1)).collect::<Vec<_>>())
            .split(centered_area);

        for (index, tab) in tabs.iter().enumerate() {
            let tab_span = Span::from(self.get_tab_text(tab)).style(if self.active == *tab {
                Style::default()
                    .bg(Color::Gray)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            });

            let line = Line::from(tab_span).alignment(HorizontalAlignment::Center);

            let area = layouts[index];

            frame.render_widget(Clear, area);
            frame.render_widget(line, area);
        }

        let block = build_block("KuberTUI", false);
        frame.render_widget(block, area);
    }

    fn get_tab_text(&self, tab: &MainWindowKind) -> String {
        match tab {
            MainWindowKind::Explorer => self.explorer_data.breadcrumb(),
            _ => tab.as_ref().to_string(),
        }
    }
}
