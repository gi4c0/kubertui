use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::Clear,
};
use strum::VariantArray;

use crate::app::MainWindowKind;

pub struct Header {
    active: MainWindowKind,
}

impl Header {
    pub fn set_active(&mut self, new_active: MainWindowKind) {
        self.active = new_active;
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame) {
        let tabs = MainWindowKind::VARIANTS;

        let layouts = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(tabs.iter().map(|_| Constraint::Fill(1)).collect::<Vec<_>>())
            .split(area);

        for (index, tab) in tabs.iter().enumerate() {
            let tab_span = Span::from(tab.as_ref()).style(if self.active == *tab {
                Style::default()
                    .bg(Color::Gray)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Black)
            });

            let area = layouts[index];

            frame.render_widget(Clear, area);
            frame.render_widget(tab_span, area);
        }
    }
}
