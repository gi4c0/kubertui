use crate::{
    app::{
        cache::{NamespaceItemCache, NamespacesListCache},
        common::{FilterableList, HelpMenuEnum, ListEvent, ListItemTrait, Spinner},
        events::{AppEvent, EventSender, KeyEventResult},
        modal::Modal,
        notification::Notification,
    },
    kubectl::pods::get_pods_list,
};
use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
};

#[derive(Debug, Clone)]
pub struct NamespacesList {
    namespace_list: FilterableList<NamespaceItem>,
    event_sender: EventSender,
}

#[derive(Debug, Clone)]
pub struct NamespaceItem {
    value: String,
    spinner: Option<Spinner>,
}

impl NamespacesList {
    pub fn new(event_sender: EventSender) -> Self {
        Self {
            event_sender,
            namespace_list: FilterableList::new("Namespaces".to_string())
                .filterable()
                .scrollable(),
        }
    }

    pub fn draw<'a>(&'a mut self, area: Rect, frame: &mut Frame) {
        self.namespace_list
            .draw_with_title(area, frame, "Namespaces");
    }

    pub fn from_cache(list: NamespacesListCache, event_sender: EventSender) -> Self {
        Self {
            event_sender,
            namespace_list: list.namespace_list.into(),
        }
    }

    pub fn set_namespaces(&mut self, namespaces: Vec<String>) {
        self.namespace_list.set_items(
            namespaces
                .into_iter()
                .map(|namespace| NamespaceItem {
                    spinner: None,
                    value: namespace,
                })
                .collect(),
        );
    }

    pub fn add_namespace(&mut self, namespace: String) {
        let existing_index = self
            .namespace_list
            .inner_list
            .iter()
            .position(|i| i.value.as_str() == namespace.as_str());

        if let Some(existing_index) = existing_index {
            self.namespace_list.inner_list.remove(existing_index);
        }

        self.namespace_list.append_to_list(NamespaceItem {
            spinner: None,
            value: namespace,
        });
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> KeyEventResult {
        if let Some(list_event) = self.namespace_list.handle_key(key) {
            match list_event {
                ListEvent::Quit => {
                    self.event_sender.send(AppEvent::Quit);
                }
                ListEvent::SelectedItem(item) => self.load_pods(item),
                ListEvent::StayInList => {}
            };
            return KeyEventResult::Consumed;
        }

        if let KeyCode::Char('?') = key.code {
            self.event_sender
                .send(AppEvent::OpenModal(Modal::help(HelpMenuEnum::Namespaces)));
            return KeyEventResult::Consumed;
        };

        KeyEventResult::Ignored
    }

    pub fn load_pods(&mut self, namespace_item: NamespaceItem) {
        let namespace_from_list = self
            .namespace_list
            .inner_list
            .iter_mut()
            .find(|namespace| namespace.value.as_str() == namespace_item.value.as_str())
            .unwrap();

        let mut spinner = Spinner::new();

        namespace_from_list.spinner = Some(spinner.clone());
        let namespace = namespace_item.value.clone();
        let event_sender = self.event_sender.clone();

        tokio::spawn(async move {
            let pods = match get_pods_list(namespace.as_str()).await {
                Ok(pods) => pods,
                Err(err) => {
                    event_sender.send(AppEvent::ShowNotification(Notification::error(err)));
                    spinner.stop();
                    return;
                }
            };

            event_sender.send(AppEvent::ShowPods { pods, namespace });
            spinner.stop();
        });
    }
}

impl From<NamespaceItem> for NamespaceItemCache {
    fn from(value: NamespaceItem) -> Self {
        Self { value: value.value }
    }
}

impl From<NamespaceItemCache> for NamespaceItem {
    fn from(value: NamespaceItemCache) -> Self {
        Self {
            value: value.value,
            spinner: None,
        }
    }
}

impl ListItemTrait for NamespaceItem {
    fn as_ref(&self) -> &str {
        self.value.as_str()
    }

    fn spinner(&self) -> Option<String> {
        self.spinner
            .as_ref()
            .and_then(|spinner| spinner.get_spin_state().map(|spinner| spinner.to_string()))
    }
}

impl From<NamespacesList> for NamespacesListCache {
    fn from(value: NamespacesList) -> Self {
        Self {
            namespace_list: value.namespace_list.into(),
        }
    }
}
