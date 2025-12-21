pub mod namespaces_list;

use std::io;

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
};

use crate::{app::namespaces_list::NamespacesList, error::AppResult, kubectl::namespace};

#[derive(Default)]
pub struct App {
    namespaces: NamespacesList,
    exit: bool,
}

impl App {
    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> AppResult<()> {
        let namespaces = namespace::get_namespaces().await.unwrap();
        self.namespaces.update_list(namespaces);

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        self.namespaces.draw(frame);
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

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Char('j') | KeyCode::Down => self.namespaces.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.namespaces.select_prev(),
            _ => {}
        };
    }
}
