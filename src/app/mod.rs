mod events;
mod namespaces_list;
mod side_bar;

use anyhow::Context;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{Event, KeyEvent, KeyEventKind},
    layout::{Constraint, Direction, Layout},
    style::Color,
};

use crate::{
    app::{
        events::{AppEvent, EventHandler},
        namespaces_list::NamespacesList,
        side_bar::SideBar,
    },
    error::AppResult,
    kubectl::namespace,
};

pub const FOCUS_COLOR: Color = Color::Cyan;

enum ActiveWindow {
    Main(MainWindow),
    RecentNamespaces,
    RecentPortForwarding,
}

enum MainWindow {
    Namespaces,
    Pods,
}

pub struct App {
    namespaces: NamespacesList,
    side_bar: SideBar,
    exit: bool,
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
        self.namespaces.draw(layouts[1], frame);
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
                self.side_bar.recent_namespaces.add_to_list(new_namespace);
            }
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        match &self.active_window {
            ActiveWindow::Main(main) => match main {
                MainWindow::Namespaces => self.namespaces.handle_key_event(key),
                MainWindow::Pods => {}
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
            active_window: ActiveWindow::Main(MainWindow::Namespaces),
            namespaces: NamespacesList::new(event_handler.sender()),
            side_bar: SideBar::new(event_handler.sender()),
            exit: false,
            event_handler,
        }
    }
}
