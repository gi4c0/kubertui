use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect};
use serde_json::Value;

use crate::{
    app::{
        common::{FilterableList, ListEvent, traits::ListItemTrait},
        events::{AppEvent, EventSender, KeyEventResult},
        main_window::{NamespacePod, logs::pod_logs::PodLogs},
        notification::Notification,
    },
    error::AppResult,
    kubectl,
};

pub mod log_item;
pub mod pod_logs;

#[derive(Debug, Clone)]
struct RecentPod {
    namespace: String,
    pod_name: String,
    title: String,
    logs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Logs {
    event_sender: EventSender,
    recent_pods: FilterableList<RecentPod>,
    pod_logs: Option<PodLogs>,
}

impl ListItemTrait for RecentPod {
    fn as_ref(&self) -> &str {
        &self.title
    }
}

impl Logs {
    pub fn new(event_sender: EventSender) -> Self {
        Self {
            event_sender,
            recent_pods: FilterableList::new("Recent Pods".to_string()),
            pod_logs: None,
        }
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame) {
        match self.pod_logs.as_mut() {
            Some(pod_logs) => pod_logs.draw(area, frame),
            None => self.recent_pods.draw(area, frame),
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> KeyEventResult {
        if let Some(current_pod) = self.pod_logs.as_mut() {
            return current_pod.handle_key_event(key);
        }

        match self.recent_pods.handle_key(key) {
            ListEvent::SelectedItem(item) => {
                self.pod_logs = Some(PodLogs::new(
                    self.event_sender.clone(),
                    item.namespace,
                    item.pod_name,
                    item.logs,
                ));
            }
            _ => return KeyEventResult::Ignored,
        };

        KeyEventResult::Consumed
    }

    pub fn reload_logs(&self) {
        if let Some(current_pod) = self.pod_logs.as_ref() {
            let NamespacePod { namespace, pod } = current_pod.get_namespace_pod();
            let event_sender = self.event_sender.clone();

            tokio::spawn(async move {
                let namespace = namespace;
                let pod = pod;

                match Self::load_prettified_logs(namespace.as_str(), pod.as_str()).await {
                    Ok(logs) => event_sender.send(AppEvent::LogsReloaded {
                        pod_name: pod,
                        logs,
                    }),
                    Err(err) => {
                        event_sender.send(AppEvent::ShowNotification(Notification::error(err)))
                    }
                };
            });
        }
    }

    pub fn logs_reloaded(&mut self, pod_name: String, logs: Vec<String>) {
        let recent_pod = self
            .recent_pods
            .inner_list
            .iter_mut()
            .find(|pod| pod.pod_name == pod_name);

        if let Some(recent_pod) = recent_pod {
            recent_pod.logs = logs.clone();
        }

        if let Some(pod_logs) = self.pod_logs.as_mut() {
            pod_logs.logs_reloaded(pod_name.as_str(), logs);
        }
    }

    pub fn load_logs(namespace: String, pod_name: String, event_sender: EventSender) {
        tokio::spawn(async move {
            let logs = match Self::load_prettified_logs(&namespace, &pod_name).await {
                Ok(logs) => Some(logs),
                Err(err) => {
                    event_sender.send(AppEvent::ShowNotification(Notification::error(err)));
                    None
                }
            };

            event_sender.send(AppEvent::LogsLoaded {
                namespace,
                pod_name,
                logs,
            });
        });
    }

    pub fn add_pod_logs(&mut self, namespace: String, pod_name: String, logs: Vec<String>) {
        let recent_pod = RecentPod {
            logs,
            title: format!("[{namespace}] {pod_name}"),
            namespace,
            pod_name,
        };

        self.pod_logs = Some(PodLogs::new(
            self.event_sender.clone(),
            recent_pod.namespace.clone(),
            recent_pod.pod_name.clone(),
            recent_pod.logs.clone(),
        ));

        self.recent_pods.push(recent_pod);
    }

    async fn load_prettified_logs(namespace: &str, pod_name: &str) -> AppResult<Vec<String>> {
        let mut logs = kubectl::load_logs(namespace, pod_name).await?;
        logs.reverse();

        let prettified_logs: Vec<String> = logs
            .into_iter()
            .map(|log| {
                let parsed: Value = match serde_json::from_str(log.as_str()) {
                    Ok(value) => value,
                    _ => {
                        return log;
                    }
                };

                serde_json::to_string_pretty(&parsed).unwrap_or(log)
            })
            .collect();

        Ok(prettified_logs)
    }
}
