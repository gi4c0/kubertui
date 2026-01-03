use crossterm::event::{KeyCode, KeyEvent};

use crate::app::{
    ActiveWindow, MainWindow, SideBarWindow,
    events::{AppEvent, EventSender},
};

pub fn handle_general_keys(key: KeyEvent, event_sender: &EventSender) -> bool {
    match key.code {
        KeyCode::Char('q') => event_sender.send(AppEvent::Quit),
        KeyCode::Char('1') => event_sender.send(AppEvent::Focus(ActiveWindow::SideBar(
            SideBarWindow::RecentNamespaces,
        ))),
        KeyCode::Char('2') => event_sender.send(AppEvent::Focus(ActiveWindow::SideBar(
            SideBarWindow::RecentPortForwards,
        ))),
        KeyCode::Char('3') => {
            event_sender.send(AppEvent::Focus(ActiveWindow::Main(MainWindow::Namespaces)))
        }

        _ => return false,
    };

    true
}
