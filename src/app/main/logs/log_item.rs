use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::Rect,
    text::Line,
    widgets::{Clear, Paragraph, Wrap},
};
use serde_json::Value;

use crate::app::common::build_block;

#[derive(Debug, Clone)]
pub enum LogItemEnum {
    PlainText(String),
    Json(Value),
}

#[derive(Debug, Clone)]
pub struct LogItem {
    text: LogItemEnum,
    pod_name: String,
    scroll: u16,
}

impl LogItem {
    pub fn new(log_item: LogItemEnum, pod_name: String) -> Self {
        Self {
            text: log_item,
            pod_name,
            scroll: 0,
        }
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame, is_focused: bool) {
        let mut paragraph = match &self.text {
            LogItemEnum::PlainText(text) => Paragraph::new(text.as_str()).wrap(Wrap { trim: true }),

            LogItemEnum::Json(json) => {
                if let Some(object) = json.as_object() {
                    let lines: Vec<Line> = object
                        .iter()
                        .map(|(key, value)| Line::from(format!("{key}: {value}")))
                        .collect();

                    Paragraph::new(lines).wrap(Wrap { trim: true })
                } else {
                    Paragraph::new(json.to_string()).wrap(Wrap { trim: true })
                }
            }
        };

        paragraph = paragraph
            .block(build_block(self.pod_name.as_str(), is_focused))
            .scroll((self.scroll, 0));

        frame.render_widget(Clear, area);
        frame.render_widget(paragraph, area);
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => return true,
            KeyCode::Char('j') | KeyCode::Down => self.scroll += 1,
            KeyCode::Char('k') | KeyCode::Up => {
                if self.scroll > 0 {
                    self.scroll -= 1;
                }
            }
            _ => {}
        }

        false
    }
}
