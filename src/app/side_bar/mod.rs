pub mod namespaces;
pub mod port_forwards_list;

use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum SideBarWindow {
    Namespaces,
    RecentPortForwards,
}

use crate::{
    app::{
        cache::SideBarCache,
        events::EventSender,
        side_bar::{namespaces::Namespaces, port_forwards_list::PortForwardsList},
    },
    error::AppResult,
};

#[derive(Clone, Debug)]
pub struct SideBar {
    pub namespaces: Namespaces,
    pub port_forwards: PortForwardsList,
}

impl From<SideBar> for SideBarCache {
    fn from(value: SideBar) -> Self {
        Self {
            namespaces: value.namespaces.into(),
            port_forwards: value.port_forwards.into(),
        }
    }
}

impl SideBar {
    pub fn initial_load(&mut self) -> AppResult<()> {
        self.namespaces.initial_load()
    }

    pub fn from_cache(value: SideBarCache, event_sender: EventSender) -> Self {
        Self {
            port_forwards: PortForwardsList::from_cache(value.port_forwards, event_sender.clone()),
            namespaces: Namespaces::from_cache(value.namespaces, event_sender.clone()),
        }
    }

    pub fn new(event_sender: EventSender) -> Self {
        Self {
            namespaces: Namespaces::new(event_sender.clone()),
            port_forwards: PortForwardsList::new(event_sender),
        }
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame, focus: Option<SideBarWindow>) {
        let layouts = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        self.namespaces
            .draw(layouts[0], frame, focus == Some(SideBarWindow::Namespaces));

        self.port_forwards.draw(
            layouts[1],
            frame,
            focus == Some(SideBarWindow::RecentPortForwards),
        );
    }

    pub fn handle_key_event(&mut self, key: KeyEvent, side_bar: SideBarWindow) {
        match side_bar {
            SideBarWindow::Namespaces => self.namespaces.handle_key_event(key),
            SideBarWindow::RecentPortForwards => self.port_forwards.handle_key_event(key),
        };
    }
}
