pub mod port_forwards;
mod recent_namespaces;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

use crate::app::{
    cache::SideBarCache,
    events::EventSender,
    side_bar::{port_forwards::PortForwardsList, recent_namespaces::RecentNamespacesList},
};

#[derive(Clone, Debug)]
pub struct SideBar {
    pub recent_namespaces: RecentNamespacesList,
    pub port_forwards: PortForwardsList,
}

impl From<SideBar> for SideBarCache {
    fn from(value: SideBar) -> Self {
        Self {
            recent_namespaces: value.recent_namespaces.into(),
            port_forwards: value.port_forwards.into(),
        }
    }
}

impl SideBar {
    pub fn new(event_sender: EventSender) -> Self {
        Self {
            recent_namespaces: RecentNamespacesList::new(event_sender),
            port_forwards: PortForwardsList::default(),
        }
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame) {
        let layouts = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        self.recent_namespaces.draw(layouts[0], frame);
        self.port_forwards.draw(layouts[1], frame);
    }
}
