use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders},
};

pub fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((area.width - width) / 2),
            Constraint::Length(width),
            Constraint::Length((area.width - width) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((area.height - height) / 2),
            Constraint::Length(height),
            Constraint::Length((area.height - height) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub const FOCUS_COLOR: Color = Color::Cyan;

pub fn build_block(title: &str) -> Block {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
}

pub fn get_highlight_style() -> Style {
    Style::default()
        .bg(FOCUS_COLOR)
        .fg(Color::Black)
        .add_modifier(Modifier::BOLD)
}
