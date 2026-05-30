use ratatui::{
    Frame,
    layout::{Constraint, Direction, Flex, HorizontalAlignment, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Clear,
};
use serde::{Deserialize, Serialize};
use strum::VariantArray;

use crate::app::{MainWindowKind, common::build_block, main::pods::PodsKind};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Header {
    active: MainWindowKind,
    pods_kind: PodsKind,
}

impl Header {
    pub fn new() -> Self {
        Self {
            active: MainWindowKind::Clusters,
            pods_kind: PodsKind::List,
        }
    }

    pub fn set_active(&mut self, new_active: MainWindowKind) {
        self.active = new_active;
    }

    pub fn set_pods_kind(&mut self, pods_kind: PodsKind) {
        self.pods_kind = pods_kind;
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

    fn get_tab_text<'a>(&self, tab: &'a MainWindowKind) -> &'a str {
        match tab {
            MainWindowKind::Pods => match self.pods_kind {
                PodsKind::Logs => "Pods -> Logs",
                PodsKind::Info => "Pods -> Info",
                _ => tab.as_ref(),
            },
            _ => tab.as_ref(),
        }
    }
}
