use crossterm::event::{KeyCode, KeyEvent};

use crate::app::{
    ActiveWindow, SideBarWindow,
    events::{AppEvent, EventSender},
};

pub fn handle_general_keys(key: KeyEvent, event_sender: &EventSender) -> bool {
    match key.code {
        KeyCode::Char('q') => event_sender.send(AppEvent::Quit),
        KeyCode::Char('1') => event_sender.send(AppEvent::Focus(ActiveWindow::SideBar(
            SideBarWindow::Namespaces,
        ))),
        KeyCode::Char('2') => event_sender.send(AppEvent::Focus(ActiveWindow::SideBar(
            SideBarWindow::RecentPortForwards,
        ))),
        KeyCode::Char('3') => event_sender.send(AppEvent::Focus(ActiveWindow::Main)),
        KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => {
            event_sender.send(AppEvent::FocusNext)
        }
        KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => {
            event_sender.send(AppEvent::FocusPrev)
        }

        _ => return false,
    };

    true
}
