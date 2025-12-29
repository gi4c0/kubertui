use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Alignment,
    widgets::Paragraph,
};

use crate::{
    app::{
        cache::PortForwardPopupCache,
        common::{FOCUS_COLOR, FilterableList, ListEvent, build_block, centered_rect},
    },
    kubectl::pods::PodContainer,
};

#[derive(Debug, Clone)]
pub struct PortForwardPopup {
    port: String,
    pod_containers_list: FilterableList<PodContainer>,
    selected_container: Option<PodContainer>,
}

impl From<PortForwardPopup> for PortForwardPopupCache {
    fn from(value: PortForwardPopup) -> Self {
        Self {
            port: value.port,
            pod_containers: value.pod_containers_list.into(),
            selected_container: value.selected_container,
        }
    }
}

impl From<PortForwardPopupCache> for PortForwardPopup {
    fn from(value: PortForwardPopupCache) -> Self {
        Self {
            port: value.port,
            pod_containers_list: value.pod_containers.into(),
            selected_container: value.selected_container,
        }
    }
}

pub enum PortForwardPopupAction {
    PortForward { local_port: u16, app_port: u16 },
    Quit,
}

impl PortForwardPopup {
    const ALLOWED_CHARS: [char; 10] = ['1', '2', '3', '4', '5', '6', '7', '8', '9', '0'];

    pub fn containers_len(&self) -> usize {
        self.pod_containers_list.list.len()
    }

    pub fn new(pod_containers: Vec<PodContainer>) -> Self {
        let mut selected_container = None;
        let mut port = String::new();

        if pod_containers.len() == 1 {
            let container = pod_containers[0].clone();
            port = container.port.to_string();
            selected_container = Some(container);
        }

        let mut list = FilterableList::new("Select container".to_string(), false);
        list.set_items(pod_containers);

        Self {
            port,
            pod_containers_list: list,
            selected_container,
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        if let Some(container) = &self.selected_container {
            let title = &format!("Forward to {}:{}", container.name.as_str(), container.port);
            let block = build_block(title.as_str())
                .title_alignment(Alignment::Center)
                .border_style(FOCUS_COLOR);

            let enter_port_widget = Paragraph::new(self.port.as_str()).block(block);
            let area = centered_rect(frame.area(), 30, 3);
            frame.render_widget(enter_port_widget, area);

            return;
        }

        let area = centered_rect(frame.area(), 30, self.containers_len() as u16 + 3);
        self.pod_containers_list.draw(area, frame, true);
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

        if let Some(list_event) = self.pod_containers_list.handle_key(key) {
            match list_event {
                ListEvent::Quit => {
                    return Some(PortForwardPopupAction::Quit);
                }
                ListEvent::SelectedItem(item) => {
                    self.port = item.port.to_string();
                    self.selected_container = Some(item);
                }
            };
        }

        None
    }
}
