pub mod cache;
mod common;
mod events;
mod namespaces_list;
mod notification;
mod pods_list;
mod side_bar;

use anyhow::Context;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{Event, KeyEvent, KeyEventKind},
    layout::{Constraint, Direction, Layout},
};
use serde::{Deserialize, Serialize};

use crate::{
    app::{
        cache::AppCache,
        events::{AppEvent, EventHandler},
        namespaces_list::NamespacesList,
        notification::NotificationWidget,
        pods_list::PodsList,
        side_bar::SideBar,
    },
    error::AppResult,
    kubectl::namespace,
};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ActiveWindow {
    Main(MainWindow),
    SideBar(SideBarWindow),
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum SideBarWindow {
    RecentNamespaces,
    RecentPortForwards,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum MainWindow {
    Namespaces,
    Pods,
}

pub struct App {
    namespaces: NamespacesList,
    pods: Option<PodsList>,
    side_bar: SideBar,
    exit: bool,
    main_window: MainWindow,
    active_window: ActiveWindow,
    event_handler: EventHandler,
    notification: Option<NotificationWidget>,
}

impl App {
    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> AppResult<()> {
        let cache = cache::read_cache().await;

        match cache {
            Some(cache) => self.merge_cache(cache),
            None => {
                let namespaces = namespace::get_namespaces()
                    .await
                    .context("Failed to download namespaces")?;

                self.namespaces.update_list(namespaces);
            }
        };

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events().await?;
        }

        Ok(())
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

        match self.main_window {
            MainWindow::Namespaces => self.namespaces.draw(
                layouts[1],
                frame,
                self.active_window == ActiveWindow::Main(MainWindow::Namespaces),
            ),
            MainWindow::Pods => match &mut self.pods {
                Some(pods_list) => pods_list.draw(
                    layouts[1],
                    frame,
                    self.active_window == ActiveWindow::Main(MainWindow::Pods),
                ),
                None => self.main_window = MainWindow::Namespaces,
            },
        };

        if let Some(notification) = &mut self.notification {
            notification.draw(frame);
        }
    }

    async fn handle_events(&mut self) -> AppResult<()> {
        match self.event_handler.next().await? {
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
                    .recent_namespaces
                    .add_to_list(new_namespace.clone());

                self.pods = Some(
                    PodsList::new(self.event_handler.sender(), new_namespace)
                        .load()
                        .await?,
                );

                self.active_window = ActiveWindow::Main(MainWindow::Pods);
                self.main_window = MainWindow::Pods;
            }
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
                self.active_window = ActiveWindow::Main(MainWindow::Namespaces);
                self.pods = None;
                self.main_window = MainWindow::Namespaces;
            }
            AppEvent::ShowNotification(notification) => {
                self.notification = Some(NotificationWidget::new(
                    notification,
                    self.event_handler.sender(),
                ))
            }
            AppEvent::HideNotification => self.notification = None,
            AppEvent::Focus(active_window) => {
                self.active_window = match active_window {
                    ActiveWindow::SideBar(side_bar) => ActiveWindow::SideBar(side_bar),
                    // A little hack since we don't know what is the active window at the moment of the call
                    ActiveWindow::Main(_) => ActiveWindow::Main(self.main_window),
                }
            }
        }

        Ok(())
    }

    async fn handle_key_event(&mut self, key: KeyEvent) {
        if let Some(notification) = &mut self.notification {
            notification.handle_key_event(key);
            return;
        }

        match &self.active_window {
            ActiveWindow::Main(main) => match main {
                MainWindow::Namespaces => self.namespaces.handle_key_event(key),
                MainWindow::Pods => {
                    if let Some(pods) = &mut self.pods {
                        pods.handle_key_event(key)
                    }
                }
            },
            ActiveWindow::SideBar(side_bar) => match side_bar {
                SideBarWindow::RecentNamespaces => {
                    self.side_bar.recent_namespaces.handle_key_event(key)
                }
                SideBarWindow::RecentPortForwards => {
                    self.side_bar.port_forwards.handle_key_event(key).await
                }
            },
        }
    }

    fn merge_cache(&mut self, cache: AppCache) {
        self.active_window = cache.active_window;
        self.main_window = cache.main_window;

        self.namespaces
            .restore_from_cache(cache.namespaces.namespace_list.into());

        self.pods = cache
            .pods
            .map(|pods_cache| PodsList::from_cache(pods_cache, self.event_handler.sender()));

        self.side_bar = self
            .side_bar
            .from_cache(cache.side_bar, self.event_handler.sender());
    }
}

impl Default for App {
    fn default() -> Self {
        let event_handler = EventHandler::new();

        Self {
            main_window: MainWindow::Namespaces,
            active_window: ActiveWindow::Main(MainWindow::Namespaces),
            namespaces: NamespacesList::new(event_handler.sender()),
            side_bar: SideBar::new(event_handler.sender()),
            exit: false,
            event_handler,
            notification: None,
            pods: None,
        }
    }
}
