pub mod cache;
pub mod common;
mod events;
mod main;
mod notification;
mod side_bar;

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{Event, KeyEvent, KeyEventKind},
    layout::{Constraint, Direction, Layout},
};
use serde::{Deserialize, Serialize};

use crate::{
    app::{
        cache::AppCache,
        common::HelpMenu,
        events::{AppEvent, EventHandler},
        main::{MainWindow, MainWindowKind},
        notification::NotificationWidget,
        side_bar::{SideBar, SideBarWindow},
    },
    error::AppResult,
};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ActiveWindow {
    Main,
    SideBar(SideBarWindow),
}

pub struct App {
    main: MainWindow,
    side_bar: SideBar,
    exit: bool,
    active_window: ActiveWindow,
    event_handler: EventHandler,
    notification: Option<NotificationWidget>,
    help_menu: HelpMenu,
}

impl App {
    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> AppResult<()> {
        let cache = cache::read_cache().await;

        match cache {
            Some(cache) => self.merge_cache(cache),
            None => self.initial_load().await?,
        };

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events().await?;
        }

        Ok(())
    }

    async fn initial_load(&mut self) -> AppResult<()> {
        self.side_bar.initial_load().await
    }

    fn draw(&mut self, frame: &mut Frame) {
        let layouts = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(25), Constraint::Percentage(75)])
            .split(frame.area());

        let side_bar_focus = match self.active_window {
            ActiveWindow::SideBar(w) => Some(w),
            _ => None,
        };

        self.side_bar.draw(layouts[0], frame, side_bar_focus);
        self.main
            .draw(layouts[1], frame, self.active_window == ActiveWindow::Main);

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
            AppEvent::SelectNamespace(new_namespace) => {
                self.side_bar
                    .namespaces
                    .add_to_recent(new_namespace.clone());

                self.main.load_pods(new_namespace).await?;
                self.active_window = ActiveWindow::Main;
            }

            AppEvent::ShowLogs(logs) => self.main.show_logs(logs),

            AppEvent::PortForward {
                pod_name,
                local_port,
                app_port,
                namespace,
            } => {
                self.side_bar
                    .port_forwards
                    .add_to_list_and_port_forward(namespace, pod_name, local_port, app_port)
                    .await;
            }

            AppEvent::ClosePodsList => {
                self.active_window = ActiveWindow::SideBar(SideBarWindow::Namespaces)
            }

            AppEvent::ShowNotification(notification) => {
                self.notification = Some(NotificationWidget::new(
                    notification,
                    self.event_handler.sender(),
                ))
            }
            AppEvent::HideNotification => self.notification = None,
            AppEvent::Focus(active_window) => self.active_window = active_window,

            AppEvent::FocusNext => {
                self.active_window = match self.active_window {
                    // ActiveWindow::Main => ActiveWindow::SideBar(SideBarWindow::Namespaces),
                    ActiveWindow::SideBar(side_bar) => match side_bar {
                        SideBarWindow::Namespaces => {
                            ActiveWindow::SideBar(SideBarWindow::RecentPortForwards)
                        }
                        SideBarWindow::RecentPortForwards => {
                            ActiveWindow::SideBar(SideBarWindow::Namespaces)
                        }
                    },
                    _ => self.active_window,
                }
            }

            AppEvent::FocusPrev => {
                self.active_window = match self.active_window {
                    // ActiveWindow::Main => ActiveWindow::SideBar(SideBarWindow::RecentPortForwards),
                    ActiveWindow::SideBar(side_bar) => match side_bar {
                        SideBarWindow::Namespaces => {
                            ActiveWindow::SideBar(SideBarWindow::RecentPortForwards)
                        }
                        SideBarWindow::RecentPortForwards => {
                            ActiveWindow::SideBar(SideBarWindow::Namespaces)
                        }
                    },
                    _ => self.active_window,
                }
            }

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

        match &self.active_window {
            ActiveWindow::Main => self.main.handle_key_event(key).await,
            ActiveWindow::SideBar(side_bar) => self.side_bar.handle_key_event(key, *side_bar).await,
        }
    }

    fn merge_cache(&mut self, cache: AppCache) {
        self.active_window = cache.active_window;
        self.main = MainWindow::from_cache(cache.main, self.event_handler.sender());
        self.side_bar = SideBar::from_cache(cache.side_bar, self.event_handler.sender());
    }
}

impl Default for App {
    fn default() -> Self {
        let event_handler = EventHandler::new();

        Self {
            active_window: ActiveWindow::SideBar(SideBarWindow::Namespaces),
            help_menu: HelpMenu::default(),
            side_bar: SideBar::new(event_handler.sender()),
            exit: false,
            main: MainWindow::new(event_handler.sender()),
            notification: None,
            event_handler,
        }
    }
}
