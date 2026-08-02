use ratatui::{Frame, crossterm::event::KeyEvent, layout::Rect};

use crate::app::{
    common::{HelpMenu, HelpMenuEnum},
    main::pods::{
        logs::log_item::LogItem,
        pods_list::{
            delete_pod_alert::{DeletePodAction, DeletePodAlert},
            port_forward_popup::{PortForwardPopup, PortForwardPopupAction},
        },
    },
    notification::NotificationWidget,
};

pub enum Modal {
    Help(HelpMenu),
    Notification(NotificationWidget),
    DeletePod(DeletePodAlert),
    PortForward(PortForwardPopup),
    LogDetail(LogItem),
}

/// What the app should do after a modal handled a key.
pub enum ModalOutcome {
    Stay,
    Close,
    CloseWith(ModalAction),
}

/// A modal's final result that requires action outside the modal itself.
pub enum ModalAction {
    DeletePod {
        namespace: String,
        pod_name: String,
    },
    PortForward {
        namespace: String,
        pod_name: String,
        local_port: u16,
        app_port: u16,
    },
}

impl Modal {
    pub fn help(kind: HelpMenuEnum) -> Self {
        Modal::Help(HelpMenu::new(kind))
    }

    pub fn draw(&mut self, frame: &mut Frame, main_area: Rect) {
        match self {
            Modal::Help(menu) => menu.draw(frame),
            Modal::Notification(widget) => widget.draw(frame),
            Modal::DeletePod(alert) => alert.draw(frame),
            Modal::PortForward(popup) => popup.draw(frame),
            Modal::LogDetail(item) => item.draw(main_area, frame),
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> ModalOutcome {
        match self {
            Modal::Help(menu) => close_if(menu.handle_key_event(key)),
            Modal::Notification(widget) => close_if(widget.handle_key_event(key)),
            Modal::LogDetail(item) => close_if(item.handle_key_event(key)),

            Modal::DeletePod(alert) => match alert.handle_key_event(key) {
                Some(DeletePodAction::DeletePod) => {
                    let (namespace, pod_name) = alert.target();
                    ModalOutcome::CloseWith(ModalAction::DeletePod {
                        namespace,
                        pod_name,
                    })
                }
                Some(DeletePodAction::Cancel) => ModalOutcome::Close,
                None => ModalOutcome::Stay,
            },

            Modal::PortForward(popup) => match popup.handle_key_event(key) {
                Some(PortForwardPopupAction::PortForward {
                    local_port,
                    app_port,
                }) => {
                    let (namespace, pod_name) = popup.target();

                    ModalOutcome::CloseWith(ModalAction::PortForward {
                        namespace,
                        pod_name,
                        local_port,
                        app_port,
                    })
                }
                Some(PortForwardPopupAction::Quit) => ModalOutcome::Close,
                None => ModalOutcome::Stay,
            },
        }
    }
}

fn close_if(should_close: bool) -> ModalOutcome {
    if should_close {
        ModalOutcome::Close
    } else {
        ModalOutcome::Stay
    }
}
