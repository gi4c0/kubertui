use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::ListItem,
};

use crate::{
    error::{AppError, AppResult},
    kubectl::{self, Log, LogLevel},
};

pub struct PodLogs {
    pod_name: String,
    logs: Vec<Log>,
}

impl PodLogs {
    pub async fn load(pod_name: String) -> AppResult<Self> {
        let logs = kubectl::load_logs(pod_name.as_str()).await?;
        Ok(Self { pod_name, logs })
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame, is_focused: bool) {
        let list_items: Vec<ListItem> = self
            .logs
            .iter()
            .map(|log| {
                let level_style = match log.level {
                    LogLevel::Error => Color::LightRed,
                    LogLevel::Info => Color::LightGreen,
                    LogLevel::Warn => Color::LightYellow,
                };

                let mut lines: Vec<Line> = vec![
                    Line::default().spans([
                        Span::from("level: "),
                        Span::from(log.level.to_string()).style(level_style),
                    ]),
                    Line::default()
                        .spans(vec![Span::from("Time: "), Span::from(log.time.as_str())]),
                ];

                if let Some(request_id) = log.request_id.as_ref() {
                    lines.push(Line::default().spans(vec![
                        Span::from("request_id"),
                        Span::from(request_id.as_str()),
                    ]));
                }

                if let Some(action) = log.request_id.as_ref() {
                    lines.push(
                        Line::default()
                            .spans(vec![Span::from("action"), Span::from(action.as_str())]),
                    );
                }

                if let Some(context) = log.request_id.as_ref() {
                    lines.push(
                        Line::default()
                            .spans(vec![Span::from("context"), Span::from(context.as_str())]),
                    );
                }

                lines.push(
                    Line::default().spans(vec![Span::from("msg: "), Span::from(log.msg.as_str())]),
                );

                lines.push(Line::from("\n"));
                ListItem::new(lines)
            })
            .collect();
    }
}
