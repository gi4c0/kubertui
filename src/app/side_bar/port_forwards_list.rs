use std::{process::Command, sync::Arc, time::Duration};

use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    crossterm::event::KeyEvent,
    layout::Rect,
    style::{Color, Style},
    widgets::ListState,
};
use tokio::{sync::Mutex, time::sleep};

use crate::{
    app::{
        cache::{PortForwardCache, PortForwardsListCache},
        common::{FilterableList, ListEvent, ListItemTrait, Spinner, handle_general_keys},
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

#[derive(Default, Debug, Clone)]
pub struct PortForward {
    namespace: String,
    pod_name: String,
    local_port: u16,
    app_port: u16,
    pid: Arc<Mutex<Option<u32>>>,
    spinner: Spinner,
    item_str: String,
}

impl PartialEq for PortForward {
    fn eq(&self, other: &Self) -> bool {
        self.item_str.as_str() == other.item_str.as_str()
    }
}

impl ListItemTrait for PortForward {
    fn get_style(&self) -> Option<Style> {
        let pid = match self.pid.try_lock() {
            Ok(pid) => *pid,
            _ => None,
        };

        pid.map(|_| Color::LightGreen.into())
    }

    fn as_ref(&self) -> &str {
        &self.item_str
    }

    fn spinner(&self) -> Option<String> {
        Some(self.spinner.get_spin_state().to_owned())
    }
}

impl From<PortForward> for PortForwardCache {
    fn from(value: PortForward) -> Self {
        loop {
            let value = value.clone();
            let pid = match value.pid.try_lock() {
                Ok(pid) => *pid,
                _ => continue,
            };

            return Self {
                item_str: value.item_str,
                local_port: value.local_port,
                namespace: value.namespace,
                pod_name: value.pod_name,
                app_port: value.app_port,
                pid,
            };
        }
    }
}

impl From<PortForwardCache> for PortForward {
    fn from(value: PortForwardCache) -> Self {
        Self {
            item_str: value.item_str,
            local_port: value.local_port,
            namespace: value.namespace,
            pod_name: value.pod_name,
            app_port: value.app_port,
            pid: Arc::new(Mutex::new(value.pid)),
            spinner: Spinner::new(),
        }
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
    const CHECK_INTERVAL: Duration = Duration::from_secs(10);

    pub fn new(event_sender: EventSender) -> Self {
        let mut state = ListState::default();
        state.select(Some(1));

        Self {
            event_sender,
            list: FilterableList::new("Recent Port Forwards".to_string(), true),
        }
    }

    pub fn from_cache(value: PortForwardsListCache, event_sender: EventSender) -> Self {
        let mut state = ListState::default();
        state.select(value.list.state.selected);

        let mut list: FilterableList<PortForward> = value.list.into();
        list.inner_list = list
            .inner_list
            .into_iter()
            .map(|item| {
                if let Some(pid) = *item.pid.try_lock().unwrap() {
                    let is_active_port_forward = match Self::check_pid(pid) {
                        Ok(is_active_pid) => is_active_pid,
                        Err(err) => {
                            event_sender.send(AppEvent::ShowNotification(Notification::warn(err)));
                            false
                        }
                    };

                    let pid = if is_active_port_forward {
                        Some(pid)
                    } else {
                        None
                    };

                    return PortForward {
                        pid: Arc::new(Mutex::new(pid)),
                        ..item
                    };
                }

                item
            })
            .collect();

        Self { list, event_sender }
    }

    fn check_pid(pid: u32) -> Result<bool, String> {
        let output = match Command::new("ps")
            .args(["-p".to_string(), pid.to_string()])
            .output()
        {
            Ok(output) => output,
            Err(err) => return Err(err.to_string()),
        };

        if output.status.success() {
            let lines_count = String::from_utf8_lossy(&output.stdout).lines().count();
            return Ok(lines_count > 1);
        }

        let error_output = String::from_utf8_lossy(&output.stderr).to_string();

        if !error_output.is_empty() {
            return Err(error_output);
        }

        Ok(false)
    }

    pub fn add_to_list(&mut self, new_item: PortForward) {
        self.list.append_to_list(new_item);
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
            && existing.pid.lock().await.is_some()
        {
            return;
        }

        let mut spinner = Spinner::new();
        spinner.start();

        let pod = PortForward {
            pid: Arc::new(Mutex::new(None)),
            namespace,
            app_port,
            local_port,
            item_str: format!("{} -> {local_port}:{app_port}", pod_name),
            pod_name,
            spinner: spinner.clone(),
        };

        let event_sender = self.event_sender.clone();
        let cloned_pod = pod.clone();

        tokio::spawn(async move {
            Self::port_forward(event_sender, cloned_pod).await;
        });

        self.add_to_list(pod);
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame, is_focused: bool) {
        self.list.draw(area, frame, is_focused);
    }

    pub async fn handle_key_event(&mut self, key: KeyEvent) {
        // Handle inner_list keys
        if let Some(event) = self.list.handle_key(key) {
            if event == ListEvent::Quit {
                self.event_sender.send(AppEvent::Quit);
            }
            return;
        }

        if handle_general_keys(key, &self.event_sender) {
            self.list.state.select(None);
            return;
        }

        match key.code {
            KeyCode::Char('p') | KeyCode::Enter | KeyCode::Char(' ') => {
                self.toggle_port_forward().await;
            }
            KeyCode::Char('d') | KeyCode::Backspace => self.delete_item().await,
            _ => {}
        };
    }

    async fn toggle_port_forward(&mut self) {
        if let Some(selected) = self.list.state.selected() {
            {
                let mut maybe_pid = self.list.inner_list[selected].pid.lock().await;

                if let Some(pid) = *maybe_pid {
                    self.stop_port_forward(pid);
                    *maybe_pid = None;
                    return;
                }
            }

            self.list.inner_list[selected].spinner.start();

            let pod = self.list.inner_list[selected].clone();
            let event_sender = self.event_sender.clone();

            tokio::spawn(async {
                Self::port_forward(event_sender, pod).await;
            });
        }
    }

    async fn delete_item(&mut self) {
        if let Some(selected) = self.list.state.selected() {
            let pod = &self.list.inner_list[selected];

            if let Some(pid) = *pod.pid.lock().await {
                self.stop_port_forward(pid);
            }

            self.list.remove(selected);
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

    async fn port_forward(event_sender: EventSender, mut pod: PortForward) {
        let pid = match kubectl::start_port_forward(
            pod.namespace.as_str(),
            pod.pod_name.as_str(),
            pod.local_port,
            pod.app_port,
        )
        .await
        {
            Ok(pid) => pid,
            Err(err) => {
                event_sender.send(AppEvent::ShowNotification(Notification::new(
                    LogLevel::Error,
                    err.to_string(),
                )));

                pod.spinner.stop().await;
                return;
            }
        };

        pod.spinner.stop().await;

        {
            let mut pod_pid = pod.pid.lock().await;
            *pod_pid = Some(pid);
        }

        Self::run_port_forward_check_health_worker(pod.pid.clone(), pod.spinner.clone());
    }

    fn run_port_forward_check_health_worker(pid: Arc<Mutex<Option<u32>>>, mut spinner: Spinner) {
        tokio::spawn(async move {
            loop {
                {
                    let mut pid_guard = pid.lock().await;

                    let pid = match *pid_guard {
                        Some(pid) => pid,
                        None => {
                            *pid_guard = None;
                            break;
                        }
                    };

                    match Self::check_pid(pid) {
                        // do nothing and wait another interval to check once again
                        Ok(is_active) if is_active => {}
                        _ => {
                            *pid_guard = None;

                            // spinner.stop().await;
                            break;
                        }
                    };
                    // After this block guard is dropped and lock is released
                }

                sleep(Self::CHECK_INTERVAL).await;
            }
        });
    }
}
