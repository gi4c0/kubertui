use ratatui::widgets::{ListState, ScrollbarState};

use crate::app::{events::EventSender, main_window::NamespacePod};

mod draw;
pub mod keys;

#[derive(Debug, Clone)]
pub struct PodLogs {
    namespace: String,
    pod_name: String,
    event_sender: EventSender,
    logs: Vec<String>,
    state: ListState,
    scrollbar_state: ScrollbarState,
    filtered_list: Vec<usize>,
    filters: Vec<String>,
    add_new_filter_mod: bool,
    edit_filters_mod: bool,
    active_filter_index: usize,
}

impl PodLogs {
    pub fn new(
        event_sender: EventSender,
        namespace: String,
        pod_name: String,
        logs: Vec<String>,
    ) -> Self {
        Self {
            event_sender,
            namespace,
            active_filter_index: 0,
            filters: Vec::new(),
            scrollbar_state: ScrollbarState::new(logs.len()),
            add_new_filter_mod: false,
            edit_filters_mod: false,
            filtered_list: logs.iter().enumerate().map(|(index, _)| index).collect(),
            pod_name,
            state: ListState::default(),
            logs,
        }
    }

    pub fn get_namespace_pod(&self) -> NamespacePod {
        NamespacePod {
            namespace: self.namespace.clone(),
            pod: self.pod_name.clone(),
        }
    }

    pub fn logs_reloaded(&mut self, pod_name: &str, logs: Vec<String>) {
        if self.pod_name != pod_name {
            return;
        }

        let scroll_position = self.scrollbar_state.get_position();

        self.scrollbar_state = ScrollbarState::new(logs.len()).position(scroll_position);
        self.filtered_list = logs.iter().enumerate().map(|(index, _)| index).collect();
        self.logs = logs;
        self.filters = Vec::new();
        self.add_new_filter_mod = false;
        self.edit_filters_mod = false;
        self.active_filter_index = 0;
    }
}
