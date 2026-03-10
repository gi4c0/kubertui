use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    widgets::{Clear, Paragraph, Wrap},
};
use serde_json::Value;

use crate::app::{
    common::build_block,
    events::{AppEvent, EventSender},
};

#[derive(Debug, Clone)]
pub struct LogItem {
    text: String,
    pod_name: String,
    scroll: u16,
    event_sender: EventSender,
}

impl LogItem {
    pub fn new(log_item: String, pod_name: String, event_sender: EventSender) -> Self {
        let text = Self::format_log(log_item);

        Self {
            text,
            event_sender,
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

    fn format_log(log_item: String) -> String {
        if let Ok(value) = serde_json::from_str::<Value>(log_item.as_str())
            && let Some(object) = value.as_object()
        {
            let lines: Vec<String> = object
                .iter()
                .map(|(key, value)| {
                    let pretty_json =
                        Self::find_and_format_json(value).unwrap_or_else(|| value.to_string());

                    format!("{key}: {pretty_json}")
                })
                .collect();

            return lines.join("\n");
        }

        log_item
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => return true,
            KeyCode::Char('j') | KeyCode::Down => self.scroll += 1,
            KeyCode::Tab | KeyCode::BackTab => {
                self.event_sender.send(AppEvent::FocusSwitch);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.scroll > 0 {
                    self.scroll -= 1;
                }
            }
            _ => {}
        }

        false
    }

    fn find_and_format_json(value: &Value) -> Option<String> {
        let str_value: &str = value.as_str()?;

        let mut result = String::new();
        let mut cursor: usize = 0;

        loop {
            let current_str = &str_value[cursor..];

            let first_bracket_index = match current_str.chars().position(|ch| ch == '{') {
                Some(index) => index,
                None => {
                    result.push_str(current_str);
                    break;
                }
            };
            let last_bracket_index = Self::find_closing_bracket_index(current_str)?;

            if let Ok(json_value) = serde_json::from_str::<Value>(
                &current_str[first_bracket_index..last_bracket_index + 1],
            ) && let Ok(pretty) = serde_json::to_string_pretty(&json_value)
            {
                result.push_str(&current_str[0..first_bracket_index]);
                result.push('\n');

                result.push_str(&pretty);
                result.push('\n');

                if last_bracket_index == current_str.len() - 1 {
                    break;
                }

                cursor = last_bracket_index + 1;
            } else {
                result.push(current_str.chars().next()?);
                cursor += 1;
            }
        }

        Some(result)
    }

    fn find_closing_bracket_index(data: &str) -> Option<usize> {
        let mut bracket_stack = vec![];
        let mut result: Option<usize> = None;

        for (index, char) in data.chars().enumerate() {
            match char {
                '{' => {
                    bracket_stack.push('{');
                }
                '}' => {
                    bracket_stack.pop();

                    if bracket_stack.is_empty() {
                        result = Some(index);
                        break;
                    }
                }
                _ => {}
            };
        }

        result
    }
}
