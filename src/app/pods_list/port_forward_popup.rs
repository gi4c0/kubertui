use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Alignment,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
};

use crate::{
    app::common::{FOCUS_COLOR, build_block, centered_rect, get_highlight_style},
    kubectl::pods::PodContainer,
};

pub struct PortForwardPopup {
    port: String,
    pod_containers: Vec<PodContainer>,
    state: ListState,
    selected_container: Option<PodContainer>,
}

pub enum PortForwardPopupAction {
    PortForward { local_port: u16, app_port: u16 },
    Quit,
}

impl PortForwardPopup {
    const ALLOWED_CHARS: [char; 10] = ['1', '2', '3', '4', '5', '6', '7', '8', '9', '0'];

    pub fn containers_len(&self) -> usize {
        self.pod_containers.len()
    }

    pub fn new(pod_containers: Vec<PodContainer>) -> Self {
        let mut state = ListState::default();
        state.select(Some(1));

        let mut selected_container = None;
        let mut port = String::new();

        if pod_containers.len() == 1 {
            let container = pod_containers[0].clone();
            port = container.port.to_string();
            selected_container = Some(container);
        }

        Self {
            port,
            pod_containers,
            selected_container,
            state,
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        if let Some(container) = &self.selected_container {
            let title = &format!("Forward to {}:{}", container.name.as_str(), container.port);
            let block = build_block(title.as_str()).title_alignment(Alignment::Center);

            let filter_widget = Paragraph::new(self.port.as_str()).block(block);
            let area = centered_rect(frame.area(), 30, 3);
            frame.render_widget(filter_widget, area);

            return;
        }

        let list_items: Vec<ListItem> = self
            .pod_containers
            .iter()
            .map(|item| ListItem::from(item.name.as_str()))
            .collect();

        let block = build_block("Choose the container").title_alignment(Alignment::Center);

        let list = List::new(list_items)
            .block(block)
            .highlight_style(get_highlight_style());

        let area = centered_rect(frame.area(), 30, self.containers_len() as u16 + 3);
        frame.render_stateful_widget(list, area, &mut self.state);
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> Option<PortForwardPopupAction> {
        if let Some(container) = &self.selected_container {
            match key.code {
                KeyCode::Char(ch) if PortForwardPopup::ALLOWED_CHARS.contains(&ch) => {
                    self.port.push(ch);
                }
                KeyCode::Backspace => {
                    self.port.pop();
                }
                KeyCode::Enter => {
                    return Some(PortForwardPopupAction::PortForward {
                        local_port: self.port.parse().unwrap(),
                        app_port: container.port,
                    });
                }
                KeyCode::Esc => return Some(PortForwardPopupAction::Quit),
                _ => {}
            };

            return None;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                return Some(PortForwardPopupAction::Quit);
            }
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_prev(),
            KeyCode::Enter => {
                let container = self.pod_containers[self.state.selected().unwrap_or(0)].clone();
                self.port = container.port.to_string();
                self.selected_container = Some(container);
            }
            _ => {}
        };

        None
    }

    fn select_next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == self.pod_containers.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };

        self.state.select(Some(i));
    }

    fn select_prev(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.pod_containers.len() - 1
                } else {
                    i - 1
                }
            }
            None => self.pod_containers.len() - 1,
        };

        self.state.select(Some(i));
    }
}
