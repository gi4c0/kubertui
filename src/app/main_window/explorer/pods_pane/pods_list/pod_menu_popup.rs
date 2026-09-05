use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect};
use serde::{Deserialize, Serialize};
use strum::{Display, VariantArray};

use crate::app::{
    common::{FilterableList, ListEvent, centered_rect, traits::ListItemTrait},
    events::{AppEvent, EventSender, KeyEventResult, PodMenuEvent},
};

#[derive(Display, VariantArray, Debug, Serialize, Deserialize, PartialEq, Copy, Clone)]
pub enum PodMenuItem {
    Info,

    #[strum(serialize = "Env Variables")]
    EnvVars,

    #[strum(serialize = "Port Forward")]
    PortForward,

    #[strum(serialize = "Delete Pod")]
    DeletePod,
}

impl ListItemTrait for PodMenuItem {}

#[derive(Debug, Clone)]
pub struct PodMenuPopup {
    event_sender: EventSender,
    list: FilterableList<PodMenuItem>,
}

impl PodMenuPopup {
    pub fn new(event_sender: EventSender) -> Self {
        let mut list = FilterableList::new(String::from("Pod Menu"));
        list.set_items(PodMenuItem::VARIANTS.to_vec());

        Self { event_sender, list }
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame) {
        let height = PodMenuItem::VARIANTS.len() + 2;

        self.list
            .draw(centered_rect(area, 30, height as u16), frame);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> KeyEventResult {
        match self.list.handle_key(key) {
            ListEvent::Quit => self.event_sender.send(AppEvent::Quit),
            ListEvent::SelectedItem(item) => {
                self.event_sender.send(PodMenuEvent::SelectedItem(item))
            }
            _ => return KeyEventResult::Ignored,
        };

        KeyEventResult::Consumed
    }
}
