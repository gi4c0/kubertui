pub mod port_forwards;
mod recent_namespaces;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

use crate::app::{
    SideBarWindow,
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
    pub fn from_cache(&mut self, value: SideBarCache, event_sender: EventSender) -> Self {
        Self {
            port_forwards: self
                .port_forwards
                .build_from_cache(value.port_forwards, event_sender.clone()),

            recent_namespaces: RecentNamespacesList::from_cache(
                value.recent_namespaces,
                event_sender.clone(),
            ),
        }
    }

    pub fn new(event_sender: EventSender) -> Self {
        Self {
            recent_namespaces: RecentNamespacesList::new(event_sender.clone()),
            port_forwards: PortForwardsList::new(event_sender),
        }
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame, focus: Option<SideBarWindow>) {
        let layouts = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        self.recent_namespaces.draw(
            layouts[0],
            frame,
            focus == Some(SideBarWindow::RecentNamespaces),
        );

        self.port_forwards.draw(
            layouts[1],
            frame,
            focus == Some(SideBarWindow::RecentPortForwards),
        );
    }
}
