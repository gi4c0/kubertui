use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::Alignment,
    style::{Color, Stylize},
    widgets::{Paragraph, Wrap},
};
use strum::Display;

use crate::app::{
    common::{self, build_block},
    events::{AppEvent, EventSender},
};

#[derive(Debug, Display)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
}

pub struct Notification {
    level: LogLevel,
    message: String,
}

impl Notification {
    pub fn new(level: LogLevel, message: String) -> Self {
        Self { level, message }
    }

    pub fn info(message: String) -> Self {
        Self {
            level: LogLevel::Info,
            message,
        }
    }

    pub fn warn(message: String) -> Self {
        Self {
            level: LogLevel::Warning,
            message,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            level: LogLevel::Error,
            message,
        }
    }
}

pub struct NotificationWidget {
    notification: Notification,
    event_sender: EventSender,
}

const NOTIFICATION_WIDTH: u16 = 80;

impl NotificationWidget {
    pub fn new(notification: Notification, event_sender: EventSender) -> Self {
        Self {
            notification,
            event_sender,
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let height = {
            let lines = self.notification.message.len() / NOTIFICATION_WIDTH as usize;
            let one_more_line = !self
                .notification
                .message
                .len()
                .is_multiple_of(NOTIFICATION_WIDTH as usize);

            let lines = if one_more_line { lines + 1 } else { lines };
            // for borders
            lines + 2
        };

        let centered = common::centered_rect(area, NOTIFICATION_WIDTH, height as u16);
        let log_level_str = self.notification.level.to_string();

        let block = build_block(log_level_str.as_str(), true)
            .bg(Color::DarkGray)
            .title_alignment(Alignment::Center);

        let notification_color = self.get_color();

        let paragraph = Paragraph::new(self.notification.message.as_str())
            .fg(notification_color)
            .block(block.border_style(notification_color))
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, centered);
    }

    fn get_color(&self) -> Color {
        match self.notification.level {
            LogLevel::Info => Color::White,
            LogLevel::Warning => Color::LightYellow,
            LogLevel::Error => Color::LightRed,
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter | KeyCode::Backspace | KeyCode::Esc | KeyCode::Char(' ') => {
                self.event_sender.send(AppEvent::HideNotification);
            }
            _ => {}
        }
    }
}
