use crate::app::main_window::logs::pod_logs::PodLogs;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Clear, List, ListItem, Paragraph},
};

use crate::app::common::{FOCUS_COLOR, build_block, scroll};

impl PodLogs {
    pub fn draw(&mut self, area: Rect, frame: &mut Frame) {
        let list_items: Vec<ListItem> = self
            .filtered_list
            .iter()
            .map(|index| {
                let log = &self.logs[*index];
                ListItem::new(log.as_str())
            })
            .collect();

        let block = build_block(self.pod_name.as_str(), false);
        let list = List::new(list_items).block(block).highlight_style(
            Style::default()
                .bg(Color::Gray)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

        if self.add_new_filter_mod || !self.filters.is_empty() {
            let layouts = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Length(3), Constraint::Min(1)])
                .split(area);

            let block = build_block("Filter", self.add_new_filter_mod);

            let mut filter_spans: Vec<Span> = Vec::with_capacity(self.filters.len() * 2);

            for (index, filter) in self.filters.iter().enumerate() {
                let mut span =
                    Span::from(filter).style(Style::default().bg(Color::Gray).fg(Color::Black));

                if self.edit_filters_mod && index == self.active_filter_index {
                    span = span.bg(FOCUS_COLOR);
                }

                filter_spans.push(span);
                filter_spans.push(Span::from(" "));
            }

            let filter_widget = Paragraph::new(Line::default().spans(filter_spans)).block(block);

            for area in &*layouts {
                frame.render_widget(Clear, *area);
            }

            frame.render_widget(filter_widget, layouts[0]);
            frame.render_stateful_widget(list, layouts[1], &mut self.state);
            scroll::render_scrollbar(layouts[1], frame, &mut self.scrollbar_state);
            return;
        }

        frame.render_widget(Clear, area);
        frame.render_stateful_widget(list, area, &mut self.state);
        scroll::render_scrollbar(area, frame, &mut self.scrollbar_state);
    }
}
