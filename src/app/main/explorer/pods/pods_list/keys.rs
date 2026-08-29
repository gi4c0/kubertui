use crossterm::event::{KeyCode, KeyEvent};

use crate::app::{
    common::{FilterEvent, HelpMenuEnum, Spinner},
    events::{AppEvent, KeyEventResult},
    main::explorer::{
        ExplorerKind,
        pods::pods_list::{
            PodsList, delete_pod_alert::DeletePodAlert, port_forward_popup::PortForwardPopup,
        },
    },
    modal::Modal,
};

impl PodsList {
    pub fn handle_key_event(&mut self, key: KeyEvent) -> KeyEventResult {
        if self.filter.is_active() {
            return match self.filter.handle_key(key) {
                FilterEvent::Ignored => KeyEventResult::Ignored,
                FilterEvent::Changed | FilterEvent::Closed { changed: true } => {
                    self.update_filtered_list();
                    KeyEventResult::Consumed
                }
                FilterEvent::Closed { changed: false } => {
                    self.state.select(Some(0));
                    KeyEventResult::Consumed
                }
            };
        }

        if let Some(key_handle_result) = self.movement_key_handler(key) {
            return key_handle_result;
        }

        self.general_key_handler(key)
    }

    fn general_key_handler(&mut self, key: KeyEvent) -> KeyEventResult {
        match key.code {
            KeyCode::Esc => {
                self.event_sender
                    .send(AppEvent::ShowExplorer(ExplorerKind::Namespaces));
            }

            KeyCode::Char('q') => {
                self.event_sender.send(AppEvent::Quit);
            }
            KeyCode::Char('d') => {
                if let Some(pod_name) = self.get_selected_pod_name() {
                    self.event_sender.send(AppEvent::OpenModal(Modal::DeletePod(
                        DeletePodAlert::new(self.namespace.clone(), pod_name.to_owned()),
                    )));
                }
            }
            KeyCode::Char('/') => self.filter.activate(),

            KeyCode::Char('p') => {
                if let Some(index) = self.selected_index() {
                    let pod = &self.original_list[index].pod;

                    self.event_sender
                        .send(AppEvent::OpenModal(Modal::PortForward(
                            PortForwardPopup::new(
                                self.namespace.clone(),
                                pod.name.clone(),
                                pod.containers.clone(),
                            ),
                        )));
                }
            }

            KeyCode::Char('?') => self
                .event_sender
                .send(AppEvent::OpenModal(Modal::help(HelpMenuEnum::Pods))),

            KeyCode::Char('L') => {
                if let Some(index) = self.selected_index() {
                    let pod_container = &mut self.original_list[index];

                    pod_container.spinner = Some(Spinner::new());

                    self.event_sender.send(AppEvent::LoadLogs {
                        namespace: self.namespace.clone(),
                        pod_name: pod_container.pod.name.clone(),
                    });
                }
            }
            _ => return KeyEventResult::Ignored,
        };

        KeyEventResult::Consumed
    }

    fn movement_key_handler(&mut self, key: KeyEvent) -> Option<KeyEventResult> {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_prev(),
            KeyCode::Char('G') => {
                if !self.filtered_list.is_empty() {
                    self.state.select(Some(self.filtered_list.len() - 1));
                }
            }
            KeyCode::Char('g') => {
                if !self.filtered_list.is_empty() {
                    self.state.select(Some(0));
                }
            }
            _ => return None,
        };

        Some(KeyEventResult::Consumed)
    }
}
