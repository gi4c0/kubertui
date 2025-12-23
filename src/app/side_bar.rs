use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, BorderType, Borders, Widget},
};

#[derive(Default)]
pub struct SideBar {}

impl SideBar {
    pub fn draw(&self, area: Rect, frame: &mut Frame) {
        let layouts = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let recent_namespaces_widget = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Recent Namespaces");

        let recent_port_forwarding_widget = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Recent Port Forwardings");

        frame.render_widget(recent_namespaces_widget, layouts[0]);
        frame.render_widget(recent_port_forwarding_widget, layouts[1]);
    }
}
