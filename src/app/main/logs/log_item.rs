use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Clear, Paragraph, Wrap},
};
use serde_json::Value;

use crate::app::common::build_block;

#[derive(Debug, Clone)]
pub struct LogItem {
    text: String,
    pod_name: String,
    scroll: u16,
}

impl LogItem {
    pub fn new(log_item: String, pod_name: String) -> Self {
        let text = if let Ok(value) = serde_json::from_str::<Value>(&log_item)
            && let Some(object) = value.as_object()
        {
            let lines: Vec<String> = object
                .iter()
                .map(|(key, value)| {
                    if let Some((before_json, maybe_inner_json, after_json)) =
                        Self::find_and_format_json(value)
                    {
                        format!("{key}: {before_json}\n{maybe_inner_json}\n{after_json}")
                    } else {
                        format!("{key}: {value}\n")
                    }
                })
                .collect();

            lines.join("\n")
        } else {
            log_item
        };

        Self {
            text,
            pod_name,
            scroll: 0,
        }
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame, is_focused: bool) {
        let paragraph = Paragraph::new(self.text.as_str())
            .wrap(Wrap { trim: false })
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

    fn find_and_format_json(value: &Value) -> Option<(String, String, String)> {
        if let Some(str_value) = value.as_str()
            && let Some(first_bracket_index) = str_value.chars().position(|ch| ch == '{')
            && let Some(last_bracket_index) = str_value.chars().rev().position(|ch| ch == '}')
        {
            let last_bracket_index = str_value.len() - 1 - last_bracket_index + 1;

            let before_json = &str_value[0..first_bracket_index];
            let maybe_inner_json = &str_value[first_bracket_index..last_bracket_index];
            let after_json = &str_value[last_bracket_index..];

            if let Ok(json_value) = serde_json::from_str::<Value>(maybe_inner_json)
                && let Ok(pretty) = serde_json::to_string_pretty(&json_value)
            {
                return Some((before_json.to_owned(), pretty, after_json.to_owned()));
            }
        }

        None
    }
}
