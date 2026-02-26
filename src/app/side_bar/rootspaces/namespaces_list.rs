use crate::{
    app::{
        cache::NamespacesListCache,
        common::{self, FilterableList, HelpMenuEnum, ListEvent, Spinner, handle_general_keys},
        events::{AppEvent, EventSender},
        notification::Notification,
    },
    error::AppResult,
    kubectl::namespace,
};
use crossterm::event::KeyCode;
use ratatui::{Frame, crossterm::event::KeyEvent, layout::Rect, text::Span};

#[derive(Debug, Clone)]
pub struct NamespacesList {
    cluster: String,
    namespace_list: FilterableList<String>,
    event_sender: EventSender,
    spinner: Spinner,
}

impl From<NamespacesList> for NamespacesListCache {
    fn from(value: NamespacesList) -> Self {
        Self {
            namespace_list: value.namespace_list.into(),
            cluster: value.cluster,
        }
    }
}

impl NamespacesList {
    pub fn new(event_sender: EventSender, cluster: String) -> Self {
        Self {
            cluster,
            event_sender,
            namespace_list: FilterableList::new("Namespaces".to_string(), true),
            spinner: Spinner::new(),
        }
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame, is_focused: bool) {
        if self.spinner.is_loading() {
            let area = common::centered_rect(area, 1, 1);
            let span = Span::from(self.spinner.get_spin_state());
            frame.render_widget(span, area);
            return;
        }

        self.namespace_list.draw(area, frame, is_focused);
    }

    pub fn from_cache(list: NamespacesListCache, event_sender: EventSender) -> Self {
        Self {
            event_sender,
            cluster: list.cluster,
            namespace_list: list.namespace_list.into(),
            spinner: Spinner::new(),
        }
    }

    pub async fn update_cluster(&mut self, cluster: String) -> AppResult<()> {
        self.cluster = cluster;
        self.reload_namespaces().await
    }

    async fn reload_namespaces(&mut self) -> AppResult<()> {
        self.spinner.start();
        let list = namespace::get_namespaces(self.cluster.as_str()).await?;
        self.spinner.stop().await;

        self.namespace_list.set_items(list);
        Ok(())
    }

    pub async fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        if let Some(list_event) = self.namespace_list.handle_key(key) {
            match list_event {
                ListEvent::Quit => {
                    self.event_sender.send(AppEvent::Quit);
                }
                ListEvent::SelectedItem(item) => {
                    self.event_sender.send(AppEvent::LoadPodsForNamespace(item));
                }
                ListEvent::StayInList => {}
            };
            return true;
        }

        match key.code {
            KeyCode::Char('r') => {
                if let Err(err) = self.reload_namespaces().await {
                    self.event_sender
                        .send(AppEvent::ShowNotification(Notification::error(err)));

                    return true;
                }
            }

            KeyCode::Char('?') => {
                self.event_sender
                    .send(AppEvent::ShowHelp(HelpMenuEnum::Namespaces));
                return true;
            }
            _ => {}
        };

        handle_general_keys(key, &self.event_sender)
    }
}
