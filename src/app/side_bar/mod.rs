pub mod port_forwards_list;
pub mod rootspaces;

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
        side_bar::{port_forwards_list::PortForwardsList, rootspaces::RootSpace},
    },
    error::AppResult,
};

#[derive(Clone, Debug)]
pub struct SideBar {
    pub root_space: RootSpace,
    pub port_forwards: PortForwardsList,
}

impl From<SideBar> for SideBarCache {
    fn from(value: SideBar) -> Self {
        Self {
            namespaces: value.root_space.into(),
            port_forwards: value.port_forwards.into(),
        }
    }
}

impl SideBar {
    pub async fn initial_load(&mut self) -> AppResult<()> {
        self.root_space.initial_load().await
    }

    pub async fn load_namespaces(&mut self, cluster: String) -> AppResult<()> {
        self.root_space.load_namespaces(cluster).await;
    }

    pub fn from_cache(value: SideBarCache, event_sender: EventSender) -> Self {
        Self {
            port_forwards: PortForwardsList::from_cache(value.port_forwards, event_sender.clone()),
            root_space: RootSpace::from_cache(value.namespaces, event_sender.clone()),
        }
    }

    pub fn new(event_sender: EventSender) -> Self {
        Self {
            root_space: RootSpace::new(event_sender.clone()),
            port_forwards: PortForwardsList::new(event_sender),
        }
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame, focus: Option<SideBarWindow>) {
        let layouts = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        self.root_space
            .draw(layouts[0], frame, focus == Some(SideBarWindow::Namespaces));

        self.port_forwards.draw(
            layouts[1],
            frame,
            focus == Some(SideBarWindow::RecentPortForwards),
        );
    }

    pub async fn handle_key_event(&mut self, key: KeyEvent, side_bar: SideBarWindow) {
        match side_bar {
            SideBarWindow::Namespaces => self.root_space.handle_key_event(key).await,
            SideBarWindow::RecentPortForwards => self.port_forwards.handle_key_event(key).await,
        };
    }
}
