pub mod port_forwards;
mod recent_namespaces;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, BorderType, Borders},
};

use crate::app::{
    events::EventSender,
    side_bar::{port_forwards::PortForwardsList, recent_namespaces::RecentNamespacesList},
};

pub struct SideBar {
    pub recent_namespaces: RecentNamespacesList,
    pub port_forwards: PortForwardsList,
}

impl SideBar {
    pub fn new(event_sender: EventSender) -> Self {
        Self {
            recent_namespaces: RecentNamespacesList::new(event_sender),
            port_forwards: PortForwardsList::default(),
        }
    }
}

impl SideBar {
    pub fn draw(&mut self, area: Rect, frame: &mut Frame) {
        let layouts = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let recent_port_forwarding_widget = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Recent Port Forwardings");

        self.recent_namespaces.draw(layouts[0], frame);
        frame.render_widget(recent_port_forwarding_widget, layouts[1]);
    }
}
