use std::process::Command;

use crossterm::event::KeyCode;
use ratatui::{Frame, crossterm::event::KeyEvent, layout::Rect, widgets::ListState};
use serde::{Deserialize, Serialize};

use crate::{
    app::{
        cache::PortForwardsListCache,
        common::{FilterableList, ListEvent, handle_general_keys},
        events::{AppEvent, EventSender},
        notification::{LogLevel, Notification},
    },
    kubectl,
};

#[derive(Debug, Clone)]
pub struct PortForwardsList {
    list: FilterableList<PortForward>,
    event_sender: EventSender,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortForward {
    namespace: String,
    pod_name: String,
    local_port: u16,
    app_port: u16,
    pid: Option<u32>,
    item_str: String,
}

impl AsRef<str> for PortForward {
    fn as_ref(&self) -> &str {
        &self.item_str
    }
}

impl From<PortForwardsList> for PortForwardsListCache {
    fn from(value: PortForwardsList) -> Self {
        Self {
            list: value.list.into(),
        }
    }
}

impl PortForwardsList {
    pub fn new(event_sender: EventSender) -> Self {
        let mut state = ListState::default();
        state.select(Some(1));

        Self {
            event_sender,
            list: FilterableList::new("Recent Port Forwards".to_string(), true),
        }
    }

    pub fn build_from_cache(
        &mut self,
        value: PortForwardsListCache,
        event_sender: EventSender,
    ) -> Self {
        let mut state = ListState::default();
        state.select(value.list.state.selected);

        let mut list: FilterableList<PortForward> = value.list.into();
        list.inner_list = list
            .inner_list
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
            .collect();

        Self { list, event_sender }
    }

    fn check_pid(&self, pid: u32) -> bool {
        let output = match Command::new("ps")
            .args(["-p".to_string(), pid.to_string()])
            .output()
        {
            Ok(output) => output,
            Err(err) => {
                self.event_sender
                    .send(AppEvent::ShowNotification(Notification::new(
                        LogLevel::Warning,
                        err.to_string(),
                    )));
                return false;
            }
        };

        if output.status.success() {
            let lines_count = String::from_utf8_lossy(&output.stdout).lines().count();
            return lines_count > 1;
        }

        let error_output = String::from_utf8_lossy(&output.stderr).to_string();

        if !error_output.is_empty() {
            self.event_sender
                .send(AppEvent::ShowNotification(Notification::new(
                    LogLevel::Warning,
                    error_output,
                )));
        }

        false
    }

    pub fn add_to_list(&mut self, new_item: PortForward) {
        if new_item.pid.is_some() {
            self.list.inner_list.insert(0, new_item);
        } else {
            self.list.inner_list.push(new_item);
        }
    }

    pub async fn add_to_list_and_port_forward(
        &mut self,
        namespace: String,
        pod_name: String,
        local_port: u16,
        app_port: u16,
    ) {
        if let Some(existing) = self
            .list
            .inner_list
            .iter()
            .find(|item| item.pod_name == pod_name)
            && existing.pid.is_some()
        {
            return;
        }

        let mut pod = PortForward {
            pid: None,
            namespace,
            app_port,
            local_port,
            item_str: format!("{} -> {local_port} {app_port}", pod_name),
            pod_name,
        };

        Self::port_forward(self.event_sender.clone(), &mut pod).await;
        self.add_to_list(pod);
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame, is_focused: bool) {
        self.list.draw(area, frame, is_focused);
    }

    pub async fn handle_key_event(&mut self, key: KeyEvent) {
        if handle_general_keys(key, &self.event_sender) {
            self.list.state.select(None);
            return;
        }

        // Handle inner_list keys
        if let Some(event) = self.list.handle_key(key)
            && event == ListEvent::Quit
        {
            self.event_sender.send(AppEvent::Quit);
            return;
        }

        match key.code {
            KeyCode::Char('p') | KeyCode::Enter | KeyCode::Char(' ') => {
                self.toggle_port_forward().await
            }
            KeyCode::Char('d') | KeyCode::Backspace => self.delete_item(),
            _ => {}
        };
    }

    async fn toggle_port_forward(&mut self) {
        if let Some(selected) = self.list.state.selected() {
            if let Some(pid) = self.list.inner_list[selected].pid {
                self.stop_port_forward(pid);
                self.list.inner_list[selected].pid = None;
                return;
            }

            let pod = &mut self.list.inner_list[selected];
            Self::port_forward(self.event_sender.clone(), pod).await;
        }
    }

    fn delete_item(&mut self) {
        if let Some(selected) = self.list.state.selected() {
            let pod = &self.list.inner_list[selected];

            if let Some(pid) = pod.pid {
                self.stop_port_forward(pid);
            }

            self.list.inner_list.remove(selected);
        }
    }

    fn stop_port_forward(&self, pid: u32) {
        if let Err(err) = kubectl::kill_process(pid) {
            self.event_sender
                .send(AppEvent::ShowNotification(Notification::error(
                    err.to_string(),
                )));
        }
    }

    async fn port_forward(event_sender: EventSender, pod: &mut PortForward) {
        match kubectl::start_port_forward(
            pod.namespace.as_str(),
            pod.pod_name.as_str(),
            pod.local_port,
            pod.app_port,
        )
        .await
        {
            Ok(pid) => pod.pid = Some(pid),
            Err(err) => event_sender.send(AppEvent::ShowNotification(Notification::new(
                LogLevel::Error,
                err.to_string(),
            ))),
        }
    }
}
