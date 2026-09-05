use ratatui::crossterm::event::{Event, KeyEventKind};

use crate::{
    app::{
        App, MainWindowKind, cache,
        events::{AppEvent, ExplorerEvent, LogsEvent},
        modal::Modal,
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

            AppEvent::ShowNotification(notification) => self
                .modals
                .push(Modal::Notification(NotificationWidget::new(notification))),

            AppEvent::OpenModal(modal) => self.modals.push(modal),

            AppEvent::FocusNext => self.set_active_window(self.next_focus()),
            AppEvent::FocusPrev => self.set_active_window(self.prev_focus()),

            AppEvent::Explorer(event) => {
                self.on_explorer_event(&event);
                self.main_window.handle_explorer_event(event);
            }

            AppEvent::Logs(event) => {
                self.on_logs_event(&event);
                self.main_window.handle_logs_event(event);
            }
        }

        Ok(())
    }

    fn on_explorer_event(&mut self, event: &ExplorerEvent) {
        self.header.handle_explorer_event(event);

        match event {
            ExplorerEvent::Show(_)
            | ExplorerEvent::NamespacesLoaded { .. }
            | ExplorerEvent::PodsLoaded { .. } => self.set_active_window(MainWindowKind::Explorer),

            _ => {}
        }
    }

    fn on_logs_event(&mut self, event: &LogsEvent) {
        if let LogsEvent::Loaded { pod_name, logs, .. } = event {
            self.main_window
                .handle_explorer_event(ExplorerEvent::PodLogsFinished {
                    pod_name: pod_name.clone(),
                });

            if logs.is_some() {
                self.set_active_window(MainWindowKind::Logs);
            }
        }
    }
}
