use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Alignment,
    style::Color,
    text::{Line, Span},
    widgets::{Clear, Paragraph},
};

use crate::app::common::{build_block, centered_rect};

pub enum DeletePodAction {
    DeletePod,
    Cancel,
}

#[derive(Debug, Clone)]
pub struct DeletePodAlert {
    pod_name: String,
}

impl DeletePodAlert {
    pub fn new(pod_name: String) -> Self {
        Self { pod_name }
    }

    pub fn draw(&self, frame: &mut Frame) {
        let title = "Are you sure?";
        let block = build_block(title, true)
            .title_bottom("<Enter>    <Esc>")
            .title_alignment(Alignment::Center);

        let area = centered_rect(frame.area(), self.pod_name.len() as u16 + 15, 3);

        let paragraph = {
            let line = Line::default().spans(vec![
                Span::from("Delete pod "),
                Span::from(self.pod_name.as_str()).style(Color::Green),
            ]);

            Paragraph::new(line)
        };

        let widget = paragraph.block(block).centered();

        frame.render_widget(Clear, area);
        frame.render_widget(widget, area);
    }

    pub fn handle_key_event(&self, key: KeyEvent) -> Option<DeletePodAction> {
        match key.code {
            KeyCode::Enter => Some(DeletePodAction::DeletePod),
            KeyCode::Esc => Some(DeletePodAction::Cancel),
            _ => None,
        }
    }
}
