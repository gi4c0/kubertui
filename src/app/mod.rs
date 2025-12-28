mod cache;
mod common;
mod events;
mod namespaces_list;
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
        events::{AppEvent, EventHandler},
        namespaces_list::NamespacesList,
        pods_list::PodsList,
        side_bar::{SideBar, port_forwards::PortForward},
    },
    error::AppResult,
    kubectl::namespace,
};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
enum ActiveWindow {
    Main(MainWindow),
    RecentNamespaces,
    RecentPortForwarding,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
enum MainWindow {
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
}

impl App {
    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> AppResult<()> {
        let namespaces = namespace::get_namespaces()
            .await
            .context("Failed to download context")?;
        self.namespaces.update_list(namespaces);

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

        self.side_bar.draw(layouts[0], frame);

        match self.main_window {
            MainWindow::Namespaces => self.namespaces.draw(layouts[1], frame),
            MainWindow::Pods => match &mut self.pods {
                Some(pods_list) => pods_list.draw(layouts[1], frame),
                None => self.main_window = MainWindow::Namespaces,
            },
        };
    }

    async fn handle_events(&mut self) -> AppResult<()> {
        match self.event_handler.next().await? {
            AppEvent::Crossterm(crossterm_event) => match crossterm_event {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event)
                }
                _ => {}
            },
            AppEvent::Quit => self.exit = true,
            AppEvent::SelectNamespace(new_namespace) => {
                self.side_bar
                    .recent_namespaces
                    .add_to_list(new_namespace.clone());

                self.pods = Some(PodsList::new(self.event_handler.sender(), new_namespace).await?);
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
                    .add_to_list_and_port_forward(PortForward {
                        pod_name,
                        is_active: true,
                        local_port,
                        app_port,
                        namespace,
                    })
                    .await;
            }
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        match &self.active_window {
            ActiveWindow::Main(main) => match main {
                MainWindow::Namespaces => self.namespaces.handle_key_event(key),
                MainWindow::Pods => {
                    if let Some(pods) = &mut self.pods {
                        pods.handle_key_event(key)
                    }
                }
            },
            ActiveWindow::RecentNamespaces => {}
            ActiveWindow::RecentPortForwarding => {}
        }
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
            pods: None,
        }
    }
}
