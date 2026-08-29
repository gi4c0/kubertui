use ratatui::crossterm::event::{Event, KeyEventKind};

use crate::{
    app::{
        App, MainWindowKind, cache, events::AppEvent, main::explorer::ExplorerKind, modal::Modal,
        notification::NotificationWidget,
    },
    error::AppResult,
};

impl App {
    pub async fn handle_events(&mut self) -> AppResult<()> {
        match self.event_handler.next().await? {
            AppEvent::Tick => {}
            AppEvent::Crossterm(crossterm_event) => match crossterm_event {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event)
                }
                _ => {}
            },
            AppEvent::Quit => {
                self.exit = true;
                cache::save_cache(self).await?;
            }
            AppEvent::ShowPods { pods, namespace } => {
                // TODO: Implement recency
                // self.side_bar.root_space.add_to_recent(namespace.clone());
                self.main_window.show_pods(namespace.clone(), pods);
                self.update_active_window(MainWindowKind::Explorer, Some(ExplorerKind::Pods));
                self.header.set_namespace(namespace);
            }

            AppEvent::NamespacesLoaded(cluster, namespaces) => {
                self.main_window.show_namespaces(namespaces);
                self.update_active_window(MainWindowKind::Explorer, Some(ExplorerKind::Namespaces));
                self.header.set_cluster(cluster);
            }

            AppEvent::LoadLogs {
                namespace,
                pod_name,
            } => {
                // Stay in the pods list until the logs arrive so the spinner is
                // visible; the window switches on `LogsLoaded`.
                self.main_window.load_logs(namespace, pod_name);
            }

            AppEvent::ReloadLogs => self.main_window.reload_logs(),

            AppEvent::ShowExplorer(kind) => {
                self.update_active_window(MainWindowKind::Explorer, Some(kind));
            }

            AppEvent::ClustersLoaded(clusters) => {
                self.main_window.set_clusters(clusters);
            }

            AppEvent::LogsLoaded {
                namespace,
                pod_name,
                logs,
            } => {
                self.main_window.stop_pods_spinner(&pod_name);

                if let Some(logs) = logs {
                    self.main_window.set_logs(namespace, pod_name, logs);
                    self.update_active_window(MainWindowKind::Logs, None);
                }
            }

            AppEvent::PodsUpdated { namespace, pods } => {
                self.main_window.pods_updated(&namespace, pods);
            }

            AppEvent::LogsReloaded { pod_name, logs } => {
                self.main_window.logs_reloaded(pod_name, logs);
            }

            AppEvent::ShowNotification(notification) => self
                .modals
                .push(Modal::Notification(NotificationWidget::new(notification))),

            AppEvent::OpenModal(modal) => self.modals.push(modal),

            AppEvent::Focus(active_window) => self.active_window = active_window,

            AppEvent::FocusNext => self.update_active_window(self.next_focus(), None),
            AppEvent::FocusPrev => self.update_active_window(self.prev_focus(), None),
        }

        Ok(())
    }
}
