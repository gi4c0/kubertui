use std::process::Command;

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

use crate::{
    app::{
        cache::{PortForwardsListCache, StateCache},
        common::{build_block, get_highlight_style},
        events::{AppEvent, EventSender, Log},
    },
    error::AppResult,
    kubectl,
};

#[derive(Debug, Clone)]
pub struct PortForwardsList {
    list: Vec<PortForward>,
    state: ListState,
    event_sender: EventSender,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct PortForward {
    pub namespace: String,
    pub pod_name: String,
    pub local_port: u16,
    pub app_port: u16,
    pub pid: Option<u32>,
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
    pub fn new(event_sender: EventSender) -> Self {
        let mut state = ListState::default();
        state.select(Some(1));

        Self {
            event_sender,
            list: vec![],
            state,
        }
    }

    pub fn build_from_cache(
        &mut self,
        value: PortForwardsListCache,
        event_sender: EventSender,
    ) -> Self {
        let mut state = ListState::default();
        state.select(value.state.selected);

        Self {
            list: value
                .list
                .into_iter()
                .map(|item| {
                    if let Some(pid) = item.pid {
                        let is_active_port_forward = self.check_pid(pid);

                        let pid = if is_active_port_forward {
                            Some(pid)
                        } else {
                            None
                        };

                        return PortForward { pid, ..item };
                    }

                    item
                })
                .collect(),
            state,
            event_sender,
        }
    }

    fn check_pid(&self, pid: u32) -> bool {
        let output = match Command::new("ps")
            .args(["-p".to_string(), pid.to_string()])
            .output()
        {
            Ok(output) => output,
            Err(err) => {
                self.event_sender
                    .send(AppEvent::ShowNotification(Log::Warning(err.to_string())));
                return false;
            }
        };

        if output.status.success() {
            let lines_count = String::from_utf8_lossy(&output.stdout).lines().count();
            return lines_count > 1;
        }

        let error_output = String::from_utf8_lossy(&output.stderr).to_string();

        self.event_sender
            .send(AppEvent::ShowNotification(Log::Warning(error_output)));

        false
    }

    pub fn add_to_list(&mut self, new_item: PortForward) {
        if new_item.pid.is_some() {
            self.list.insert(0, new_item);
        } else {
            self.list.push(new_item);
        }
    }

    pub async fn add_to_list_and_port_forward(
        &mut self,
        namespace: String,
        pod_name: String,
        local_port: u16,
        app_port: u16,
    ) -> AppResult<()> {
        let pid = kubectl::start_port_forward(
            namespace.as_str(),
            pod_name.as_str(),
            local_port,
            app_port,
        )
        .await?;

        self.add_to_list(PortForward {
            pid: Some(pid),
            namespace,
            app_port,
            local_port,
            pod_name,
        });

        Ok(())
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

                if item.pid.is_some() {
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
            KeyCode::Char('p') | KeyCode::Enter => todo!(),
            KeyCode::Char('d') => {
                if let Some(selected) = self.state.selected() {
                    self.delete_item(selected);
                }
            }
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

    fn delete_item(&mut self, index: usize) {
        todo!()
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
