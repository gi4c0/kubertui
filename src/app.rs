pub mod cache;
pub mod common;
mod events;
mod header;
pub mod main;
pub mod modal;
mod notification;

use std::env;

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{Event, KeyEvent, KeyEventKind},
    layout::{Constraint, Direction, Layout},
};
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, VariantArray};

use crate::{
    app::{
        cache::AppCache,
        events::{AppEvent, EventHandler},
        header::Header,
        main::{
            MainWindow,
            explorer::{ExplorerKind, pods::pods_list::utils::delete_pod},
        },
        modal::{Modal, ModalAction, ModalOutcome},
        notification::NotificationWidget,
    },
    error::AppResult,
};

#[derive(Debug, Serialize, Deserialize, PartialEq, Copy, Clone, VariantArray, AsRefStr)]
pub enum MainWindowKind {
    Explorer,
    Logs,
    #[strum(serialize = "Port Forward")]
    PortForward,
}

pub struct App {
    header: Header,
    main_window: MainWindow,
    exit: bool,
    active_window: MainWindowKind,
    event_handler: EventHandler,
    modals: Vec<Modal>,
}

impl App {
    const CACHE_KEY: &'static str = "KUBERTUI_USE_CACHE";

    const FOCUS_ORDER: [MainWindowKind; 3] = [
        MainWindowKind::Explorer,
        MainWindowKind::Logs,
        MainWindowKind::PortForward,
    ];

    fn next_focus(&self) -> MainWindowKind {
        let active_index = Self::FOCUS_ORDER
            .iter()
            .position(|item| *item == self.active_window)
            .unwrap_or(0);

        *Self::FOCUS_ORDER
            .get(active_index + 1)
            .unwrap_or(Self::FOCUS_ORDER.first().unwrap())
    }

    fn prev_focus(&self) -> MainWindowKind {
        let active_index = Self::FOCUS_ORDER
            .iter()
            .position(|item| *item == self.active_window)
            .unwrap_or(0);

        if active_index == 0 {
            return *Self::FOCUS_ORDER.last().unwrap();
        }

        *Self::FOCUS_ORDER.get(active_index - 1).unwrap()
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> AppResult<()> {
        let cache_key_value = env::var(Self::CACHE_KEY).unwrap_or(String::new());

        if (cache_key_value.as_str() == "1" || cache_key_value.as_str() == "true")
            && let Some(cache) = cache::read_cache().await
        {
            self.merge_cache(cache);
        } else {
            self.initial_load().await?;
        }

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events().await?;
        }

        Ok(())
    }

    async fn initial_load(&mut self) -> AppResult<()> {
        self.main_window.initial_load().await
    }

    fn draw(&mut self, frame: &mut Frame) {
        let layouts = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(3), Constraint::Fill(1)])
            .split(frame.area());

        self.header.draw(layouts[0], frame);
        self.main_window.draw(layouts[1], frame);

        for modal in &mut self.modals {
            modal.draw(frame, layouts[1]);
        }
    }

    async fn handle_events(&mut self) -> AppResult<()> {
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
                self.main_window.show_pods(namespace, pods);
                self.update_active_window(MainWindowKind::Explorer, Some(ExplorerKind::Pods));
            }

            AppEvent::LoadNamespaces(namespaces) => {
                self.main_window.show_namespaces(namespaces);
                self.update_active_window(MainWindowKind::Explorer, Some(ExplorerKind::Namespaces));
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

    fn handle_key_event(&mut self, key: KeyEvent) {
        let Some(modal) = self.modals.last_mut() else {
            self.main_window.handle_key_event(key);
            return;
        };

        match modal.handle_key_event(key) {
            ModalOutcome::Stay => {}
            ModalOutcome::Close => {
                self.modals.pop();
            }
            ModalOutcome::CloseWith(action) => {
                self.modals.pop();
                self.handle_modal_action(action);
            }
        }
    }

    fn handle_modal_action(&mut self, action: ModalAction) {
        match action {
            ModalAction::DeletePod {
                namespace,
                pod_name,
            } => delete_pod(namespace, pod_name, self.event_handler.sender()),

            ModalAction::PortForward {
                namespace,
                pod_name,
                local_port,
                app_port,
            } => self
                .main_window
                .add_to_list_and_port_forward(namespace, pod_name, local_port, app_port),
        }
    }

    fn merge_cache(&mut self, cache: AppCache) {
        self.active_window = cache.active_window;
        self.main_window = MainWindow::from_cache(cache.main_window, self.event_handler.sender());
        self.header = cache.header;
    }

    fn update_active_window(
        &mut self,
        new_active_window: MainWindowKind,
        explorer_kind: Option<ExplorerKind>,
    ) {
        self.active_window = new_active_window;
        self.main_window
            .set_active_window(new_active_window, explorer_kind);
        self.header.set_active(new_active_window);

        if let Some(kind) = explorer_kind {
            self.header.set_explorer_kind(kind);
        }
    }
}

impl Default for App {
    fn default() -> Self {
        let event_handler = EventHandler::new();

        Self {
            active_window: MainWindowKind::Explorer,
            exit: false,
            main_window: MainWindow::new(event_handler.sender()),
            modals: Vec::new(),
            event_handler,
            header: Header::new(),
        }
    }
}
