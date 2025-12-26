use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Alignment, Rect},
    style::Style,
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::{app::FOCUS_COLOR, kubectl::pods::PodContainer};

pub struct PortForwardPopup {
    port: String,
    pod_containers: Vec<PodContainer>,
}

pub enum PortForwardPopupAction {
    PortForward { local_port: u16, app_port: u16 },
    Quit,
}

impl PortForwardPopup {
    const ALLOWED_CHARS: [char; 10] = ['1', '2', '3', '4', '5', '6', '7', '8', '9', '0'];

    pub fn new(pod_containers: Vec<PodContainer>) -> Self {
        Self {
            port: String::new(),
            pod_containers,
        }
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame) {
        // TODO: handle multiple containers
        let container = &self.pod_containers[0];

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(
                "Forward to {}:{}",
                container.name.as_str(),
                container.port
            ))
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .border_style(FOCUS_COLOR);

        let filter_widget = Paragraph::new(self.port.as_str()).block(block);

        frame.render_widget(filter_widget, area);
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> Option<PortForwardPopupAction> {
        match key.code {
            KeyCode::Char(ch) if PortForwardPopup::ALLOWED_CHARS.contains(&ch) => {
                self.port.push(ch);
            }
            KeyCode::Backspace => {
                self.port.pop();
            }
            KeyCode::Enter => {
                // TODO: handle multiple containers
                return Some(PortForwardPopupAction::PortForward {
                    local_port: self.port.parse().unwrap(),
                    app_port: self.pod_containers[0].port,
                });
            }
            KeyCode::Esc => return Some(PortForwardPopupAction::Quit),
            _ => {}
        };

        None
    }
}
