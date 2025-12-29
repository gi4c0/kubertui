use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    crossterm::event::KeyEvent,
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::{List, ListItem, ListState},
};
use serde::{Deserialize, Serialize};

use crate::app::{
    cache::{PortForwardsListCache, StateCache},
    common::{build_block, get_highlight_style},
};

#[derive(Default, Debug, Clone)]
pub struct PortForwardsList {
    list: Vec<PortForward>,
    state: ListState,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct PortForward {
    pub namespace: String,
    pub pod_name: String,
    pub is_active: bool,
    pub local_port: u16,
    pub app_port: u16,
}

impl From<PortForwardsList> for PortForwardsListCache {
    fn from(value: PortForwardsList) -> Self {
        Self {
            list: value.list,
            state: StateCache {
                selected: value.state.selected(),
            },
        }
    }
}

impl PortForwardsList {
    pub fn from_cache(value: PortForwardsListCache) -> Self {
        let mut state = ListState::default();
        state.select(value.state.selected);

        Self {
            list: value.list,
            state,
        }
    }

    pub fn add_to_list(&mut self, new_item: PortForward) {
        if new_item.is_active {
            self.list.insert(0, new_item);
        } else {
            self.list.push(new_item);
        }
    }

    pub async fn add_to_list_and_port_forward(&mut self, new_item: PortForward) {
        // kubectl::start_port_forward(
        //     new_item.namespace.as_str(),
        //     new_item.pod_name.as_str(),
        //     new_item.local_port,
        //     new_item.app_port,
        // )
        // .await
        // .unwrap();

        self.add_to_list(new_item);
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame) {
        let namespaces_list_items: Vec<ListItem> = self
            .list
            .iter()
            .map(|item| {
                let mut span = Span::from(format!(
                    "{} {} -> {}",
                    item.pod_name, item.local_port, item.app_port
                ));

                if item.is_active {
                    span = span.style(Style::default().fg(Color::Green));
                }

                ListItem::new(span)
            })
            .collect();

        let list = List::new(namespaces_list_items)
            .block(build_block("Port Forwards"))
            .highlight_style(get_highlight_style());

        frame.render_stateful_widget(list, area, &mut self.state);
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_prev(),
            KeyCode::Char('p') | KeyCode::Enter => {}
            _ => {}
        }
    }

    fn select_next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == self.list.len() - 1 {
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
                    self.list.len() - 1
                } else {
                    i - 1
                }
            }
            None => self.list.len() - 1,
        };

        self.state.select(Some(i));
    }
}
