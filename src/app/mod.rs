pub mod cache;
pub mod common;
mod events;
mod header;
mod main;
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
        common::HelpMenu,
        events::{AppEvent, EventHandler},
        header::Header,
        main::MainWindow,
        notification::NotificationWidget,
    },
    error::AppResult,
};

#[derive(Debug, Serialize, Deserialize, PartialEq, Copy, Clone, VariantArray, AsRefStr)]
pub enum MainWindowKind {
    Clusters,
    Namespaces,
    Pods,
    #[strum(serialize = "Port Forward")]
    PortForward,
}

pub struct App {
    header: Header,
    main_window: MainWindow,
    exit: bool,
    active_window: MainWindowKind,
    event_handler: EventHandler,
    notification: Option<NotificationWidget>,
    // TODO: refactor items to be executable
    help_menu: HelpMenu,
}

impl App {
    const CACHE_KEY: &'static str = "KUBERTUI_USE_CACHE";

    const FOCUS_ORDER: [MainWindowKind; 4] = [
        MainWindowKind::Clusters,
        MainWindowKind::Namespaces,
        MainWindowKind::Pods,
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
            .constraints(vec![Constraint::Length(10), Constraint::Fill(1)])
            .split(frame.area());

        self.header.draw(layouts[0], frame);
        self.main_window.draw(layouts[1], frame);

        if let Some(notification) = &mut self.notification {
            notification.draw(frame);
        }

        if self.help_menu.is_shown() {
            self.help_menu.draw(frame);
        }
    }

    async fn handle_events(&mut self) -> AppResult<()> {
        match self.event_handler.next().await? {
            AppEvent::Tick => {}
            AppEvent::Crossterm(crossterm_event) => match crossterm_event {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event).await
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
                self.active_window = MainWindowKind::Pods;
            }

            AppEvent::LoadNamespaces(namespaces) => {
                self.main_window.show_namespaces(namespaces);
                self.update_active_window(MainWindowKind::Namespaces);
            }

            AppEvent::ShowLogs(logs) => self.main_window.show_logs(logs),

            AppEvent::PortForward {
                pod_name,
                local_port,
                app_port,
                namespace,
            } => {
                self.main_window
                    .add_to_list_and_port_forward(namespace, pod_name, local_port, app_port)
                    .await;
            }

            AppEvent::ShowNotification(notification) => {
                self.notification = Some(NotificationWidget::new(
                    notification,
                    self.event_handler.sender(),
                ))
            }
            AppEvent::HideNotification => self.notification = None,
            AppEvent::Focus(active_window) => self.active_window = active_window,

            AppEvent::FocusNext => self.update_active_window(self.next_focus()),
            AppEvent::FocusPrev => self.update_active_window(self.prev_focus()),

            AppEvent::ShowHelp(kind) => {
                self.help_menu.show(kind);
            }
        }

        Ok(())
    }

    async fn handle_key_event(&mut self, key: KeyEvent) {
        if let Some(notification) = &mut self.notification {
            notification.handle_key_event(key);
            return;
        }

        if self.help_menu.is_shown() {
            self.help_menu.handle_key_event(key);
            return;
        }

        self.main_window.handle_key_event(key).await;
    }

    fn merge_cache(&mut self, cache: AppCache) {
        self.active_window = cache.active_window;
        self.main_window = MainWindow::from_cache(cache.main_window, self.event_handler.sender());
        self.header = cache.header;
    }

    pub fn update_active_window(&mut self, new_active_window: MainWindowKind) {
        self.active_window = new_active_window;
        self.main_window.update_active_window(new_active_window);
        self.header.set_active(new_active_window);
    }
}

impl Default for App {
    fn default() -> Self {
        let event_handler = EventHandler::new();

        Self {
            active_window: MainWindowKind::Clusters,
            help_menu: HelpMenu::default(),
            exit: false,
            main_window: MainWindow::new(event_handler.sender()),
            notification: None,
            event_handler,
            header: Header::new(),
        }
    }
}
