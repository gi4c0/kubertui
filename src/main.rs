use std::io;

mod error;
mod kubectl;

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::{error::AppResult, kubectl::namespace};

#[derive(Default)]
pub struct App {
    namespaces: Vec<String>,
    namespaces_state: ListState,
    exit: bool,
}

impl App {
    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> AppResult<()> {
        let namespaces = namespace::get_namespaces().await.unwrap();
        self.namespaces = namespaces;

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let namespaces_list_items: Vec<ListItem> = self
            .namespaces
            .iter()
            .map(|namespace| ListItem::new(namespace.as_str()))
            .collect();

        let list = List::new(namespaces_list_items)
            .block(
                Block::default()
                    .title("Select Namespace")
                    .borders(Borders::ALL),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_stateful_widget(list, frame.area(), &mut self.namespaces_state);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };

        Ok(())
    }

    fn next_namespace(&mut self) {
        let i = match self.namespaces_state.selected() {
            Some(i) => {
                if i == self.namespaces.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };

        self.namespaces_state.select(Some(i));
    }

    fn previous_namespace(&mut self) {
        let i = match self.namespaces_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.namespaces.len() - 1
                } else {
                    i - 1
                }
            }
            None => self.namespaces.len() - 1,
        };

        self.namespaces_state.select(Some(i));
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Char('j') | KeyCode::Down => self.next_namespace(),
            KeyCode::Char('k') | KeyCode::Up => self.previous_namespace(),
            _ => {}
        };
    }
}

#[tokio::main]
async fn main() -> AppResult<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal).await;
    ratatui::restore();
    app_result
}
