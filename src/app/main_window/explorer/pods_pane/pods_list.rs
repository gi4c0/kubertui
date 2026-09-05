pub mod delete_pod_alert;
pub mod pod_menu_popup;
pub mod port_forward_popup;
pub mod utils;

mod draw;
mod keys;

use ratatui::widgets::TableState;

use crate::{
    app::{
        cache::{PodsListCache, StateCache},
        common::{Filter, Spinner},
        events::{AppEvent, EventSender, PodMenuEvent},
        main_window::explorer::pods_pane::pods_list::{
            delete_pod_alert::DeletePodAlert,
            pod_menu_popup::{PodMenuItem, PodMenuPopup},
            port_forward_popup::PortForwardPopup,
        },
        modal::Modal,
    },
    kubectl::pods::Pod,
};

// TODO: display spinner on port forward and redirect to port forward list on success
#[derive(Debug, Clone)]
pub struct PodsList {
    original_list: Vec<PodWithSpinner>,
    filtered_list: Vec<usize>,
    event_sender: EventSender,
    state: TableState,
    filter: Filter,
    longest_name: u16,
    title: String,
    namespace: String,
    pod_menu_popup: Option<PodMenuPopup>,
}

impl From<PodsList> for PodsListCache {
    fn from(value: PodsList) -> Self {
        let (filter, is_filter_mod) = value.filter.into_parts();

        Self {
            filter,
            filtered_list: value.filtered_list,
            is_filter_mod,
            original_list: value.original_list.into_iter().map(Into::into).collect(),
            longest_name: value.longest_name,
            namespace: value.namespace,
            title: value.title,
            state: StateCache {
                selected: value.state.selected(),
            },
        }
    }
}

impl PodsList {
    pub fn from_cache(value: PodsListCache, event_sender: EventSender) -> Self {
        let mut state = TableState::default();
        state.select(value.state.selected);

        Self {
            filter: Filter::from_parts(value.filter, value.is_filter_mod),
            event_sender,
            filtered_list: value.filtered_list,
            original_list: value.original_list.into_iter().map(Into::into).collect(),
            longest_name: value.longest_name,
            title: value.title,
            namespace: value.namespace,
            state,
            pod_menu_popup: None,
        }
    }

    pub fn new(event_sender: EventSender, namespace: String, pods: Vec<Pod>) -> Self {
        let mut state = TableState::default();
        state.select(Some(0));

        let mut list = Self {
            filtered_list: Vec::new(),
            namespace: namespace.clone(),
            title: format!("[{namespace}] Select pod"),
            longest_name: 0,
            original_list: Vec::new(),
            event_sender,
            state,
            filter: Filter::default(),
            pod_menu_popup: None,
        };

        list.update_pods(pods);
        list
    }

    pub fn handle_event(&mut self, event: PodMenuEvent) {
        self.pod_menu_popup = None;

        match event {
            PodMenuEvent::CloseMenuPopup => {} // already removed popup above
            PodMenuEvent::SelectedItem(PodMenuItem::DeletePod) => self.delete_pod(),
            PodMenuEvent::SelectedItem(PodMenuItem::PortForward) => self.port_forward(),
            PodMenuEvent::SelectedItem(PodMenuItem::EnvVars) => todo!(),
            PodMenuEvent::SelectedItem(PodMenuItem::Info) => todo!(),
        }
    }

    fn port_forward(&self) {
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

    fn update_pods(&mut self, pods: Vec<Pod>) {
        let longest_name = pods
            .iter()
            .max_by_key(|p| p.name.len())
            .map(|p| p.name.len())
            .unwrap_or(10) as u16;

        self.longest_name = longest_name;
        self.filtered_list = pods.iter().enumerate().map(|(index, _)| index).collect();
        self.original_list = pods.into_iter().map(Into::into).collect();
        self.state.select(Some(0));
    }

    /// Stops by pod name, not by selection: the selection may have moved while
    /// the logs were loading.
    pub fn stop_spinner(&mut self, pod_name: &str) {
        let Some(pod_container) = self
            .original_list
            .iter_mut()
            .find(|item| item.pod.name == pod_name)
        else {
            return;
        };

        if let Some(spinner) = pod_container.spinner.as_mut() {
            spinner.stop();
        }

        pod_container.spinner = None;
    }

    /// Index into `original_list` of the selected row, if any. Safe when the
    /// filtered list is empty or the selection is out of range.
    fn selected_index(&self) -> Option<usize> {
        let selected = self.state.selected()?;
        self.filtered_list.get(selected).copied()
    }

    fn get_selected_pod_name(&self) -> Option<&str> {
        let index = self.selected_index()?;

        Some(self.original_list[index].pod.name.as_str())
    }

    pub fn pods_updated(&mut self, namespace: &str, pods: Vec<Pod>) {
        if self.namespace != namespace {
            return;
        }

        self.update_pods(pods);
        self.update_filtered_list();
    }

    fn update_filtered_list(&mut self) {
        self.filtered_list = self.filter.apply(
            self.original_list
                .iter()
                .map(|item| item.pod.name.to_string()),
        );
        self.state.select(Some(0));
    }

    fn select_next(&mut self) {
        if self.filtered_list.is_empty() {
            return self.state.select(None);
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i == self.filtered_list.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };

        self.state.select(Some(i));
    }

    fn select_prev(&mut self) {
        if self.filtered_list.is_empty() {
            return self.state.select(None);
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_list.len() - 1
                } else {
                    i - 1
                }
            }
            None => self.filtered_list.len() - 1,
        };

        self.state.select(Some(i));
    }

    fn delete_pod(&self) {
        if let Some(pod_name) = self.get_selected_pod_name() {
            self.event_sender
                .send(AppEvent::OpenModal(Modal::DeletePod(DeletePodAlert::new(
                    self.namespace.clone(),
                    pod_name.to_owned(),
                ))));
        }
    }
}

#[derive(Debug, Clone)]
struct PodWithSpinner {
    pod: Pod,
    spinner: Option<Spinner>,
}

impl From<Pod> for PodWithSpinner {
    fn from(value: Pod) -> Self {
        Self {
            pod: value,
            spinner: None,
        }
    }
}

impl From<PodWithSpinner> for Pod {
    fn from(value: PodWithSpinner) -> Self {
        Self { ..value.pod }
    }
}
