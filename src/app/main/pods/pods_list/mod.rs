pub mod delete_pod_alert;
pub mod port_forward_popup;

use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Cell, Paragraph, Row, Table, TableState},
};

use crate::{
    app::{
        cache::{PodsListCache, StateCache},
        common::{HelpMenuEnum, Spinner, build_block, get_highlight_style},
        events::{AppEvent, EventSender, KeyEventResult},
        main::pods::pods_list::{
            delete_pod_alert::DeletePodAlert, port_forward_popup::PortForwardPopup,
        },
        modal::Modal,
        notification::Notification,
    },
    kubectl::{
        self,
        pods::{KnownPodStatus, Pod, PodStatus, get_pods_list},
    },
};

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

#[derive(Debug, Clone)]
pub struct PodsList {
    original_list: Vec<PodWithSpinner>,
    filtered_list: Vec<usize>,
    event_sender: EventSender,
    state: TableState,
    filter: String,
    is_filter_mod: bool,
    longest_name: u16,
    title: String,
    namespace: String,
}

impl From<PodsList> for PodsListCache {
    fn from(value: PodsList) -> Self {
        Self {
            filter: value.filter,
            filtered_list: value.filtered_list,
            is_filter_mod: value.is_filter_mod,
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
            filter: value.filter,
            event_sender,
            filtered_list: value.filtered_list,
            is_filter_mod: value.is_filter_mod,
            original_list: value.original_list.into_iter().map(Into::into).collect(),
            longest_name: value.longest_name,
            title: value.title,
            namespace: value.namespace,
            state,
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
            filter: String::new(),
            is_filter_mod: false,
        };

        list.update_pods(pods);
        list
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

    pub fn draw(&mut self, area: Rect, frame: &mut Frame) {
        let header = ["Name", "Containers"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>();

        let rows: Vec<Row> = self
            .filtered_list
            .iter()
            .map(|index| {
                let item = &self.original_list[*index];

                let maybe_spinner = item
                    .spinner
                    .as_ref()
                    .and_then(|spinner| spinner.get_spin_state())
                    .unwrap_or(" ");

                let pod_name = format!("{maybe_spinner} {}", item.pod.name.as_str());

                Row::new([
                    pod_name.into(),
                    get_status(&item.pod.container_statuses, &item.pod.reason),
                ])
            })
            .collect();

        let block = build_block("Select pod", false);

        let table = Table::new(
            rows,
            [
                Constraint::Length(self.longest_name + 3),
                Constraint::Min(5),
            ],
        )
        .header(header)
        .block(block)
        .row_highlight_style(get_highlight_style());

        if self.is_filter_mod || !self.filter.is_empty() {
            let layouts = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Length(3), Constraint::Min(1)])
                .split(area);

            let block = build_block("Filter", self.is_filter_mod);
            let filter_widget = Paragraph::new(self.filter.as_str()).block(block);

            frame.render_widget(filter_widget, layouts[0]);
            frame.render_stateful_widget(table, layouts[1], &mut self.state);
        } else {
            frame.render_stateful_widget(table, area, &mut self.state);
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> KeyEventResult {
        if self.is_filter_mod {
            match key.code {
                KeyCode::Enter => {
                    self.is_filter_mod = false;
                    self.state.select(Some(0));
                }
                KeyCode::Esc => {
                    self.filter.clear();
                    self.is_filter_mod = false;
                    self.update_filtered_list();
                    self.state.select(Some(0));
                }
                KeyCode::Backspace => {
                    self.filter.pop();
                    self.update_filtered_list();
                }
                KeyCode::Char(ch) => {
                    self.filter.push(ch);
                    self.update_filtered_list();
                }
                _ => {}
            };

            return KeyEventResult::Consumed;
        }

        match key.code {
            KeyCode::Char('q') => {
                self.event_sender.send(AppEvent::Quit);
            }
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_prev(),
            KeyCode::Char('d') => {
                if let Some(pod_name) = self.get_selected_pod_name() {
                    self.event_sender.send(AppEvent::OpenModal(Modal::DeletePod(
                        DeletePodAlert::new(self.namespace.clone(), pod_name.to_owned()),
                    )));
                }
            }
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
            KeyCode::Char('/') => self.is_filter_mod = true,
            KeyCode::Char('p') => {
                let index = self.filtered_list[self.state.selected().unwrap_or(0)];
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

            KeyCode::Char('?') => self
                .event_sender
                .send(AppEvent::OpenModal(Modal::help(HelpMenuEnum::Pods))),

            KeyCode::Char('L') => {
                let index = self.filtered_list[self.state.selected().unwrap_or(0)];
                let pod_container = &mut self.original_list[index];

                let spinner = Spinner::new();
                pod_container.spinner = Some(spinner.clone());

                let pod_name = pod_container.pod.name.clone();

                self.event_sender.send(AppEvent::LoadLogs {
                    namespace: self.namespace.clone(),
                    pod_name,
                });
            }
            _ => return KeyEventResult::Ignored,
        };

        KeyEventResult::Consumed
    }

    pub fn stop_spinner(&mut self) {
        let index = self.filtered_list[self.state.selected().unwrap_or(0)];
        let pod_container = &mut self.original_list[index];

        if let Some(spinner) = pod_container.spinner.as_mut() {
            spinner.stop();
        }

        pod_container.spinner = None;
    }

    fn get_selected_pod_name(&self) -> Option<&str> {
        let selected = self.state.selected()?;

        let pod_name = self.original_list[self.filtered_list[selected]]
            .pod
            .name
            .as_str();

        Some(pod_name)
    }

    pub fn pods_updated(&mut self, namespace: &str, pods: Vec<Pod>) {
        if self.namespace != namespace {
            return;
        }

        self.update_pods(pods);
        self.update_filtered_list();
    }

    fn update_filtered_list(&mut self) {
        self.filtered_list = self
            .original_list
            .iter()
            .enumerate()
            .filter(|(_, item)| item.pod.name.contains(self.filter.as_str()))
            .map(|(index, _)| index)
            .collect();
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
}

pub fn delete_pod(namespace: String, pod_name: String, event_sender: EventSender) {
    tokio::spawn(async move {
        if let Err(err) = kubectl::delete_pod(&namespace, &pod_name).await {
            event_sender.send(AppEvent::ShowNotification(Notification::error(err)));
        }

        match get_pods_list(namespace.as_str()).await {
            Ok(pods) => event_sender.send(AppEvent::PodsUpdated { namespace, pods }),
            Err(err) => event_sender.send(AppEvent::ShowNotification(Notification::error(err))),
        }
    });
}

fn get_status<'a>(statuses: &'a [PodStatus], reason: &'a Option<String>) -> Cell<'a> {
    if statuses.len() <= 5 {
        let statuses: Vec<String> = statuses
            .iter()
            .map(|status| match status {
                PodStatus::Unknown(status) => "❓".into(),

                PodStatus::Known(known_status) => match known_status {
                    KnownPodStatus::Running { started_at: _ } => "💚".into(),
                    KnownPodStatus::Terminated {
                        container_id: _,
                        exit_code: _,
                        finished_at: _,
                        reason: _,
                        started_at: _,
                        message: _,
                    } => "💔".into(),
                    KnownPodStatus::Waiting {
                        reason: _,
                        message: _,
                    } => "💤".into(),
                },
            })
            .collect();

        if statuses.is_empty()
            && let Some(reason) = reason
        {
            let icon = match reason.as_str() {
                "Evicted" => "❌".to_string(),
                another => format!("❌ ({another})").to_string(),
            };

            return Cell::from(icon);
        }

        return Cell::from(statuses.join(" "));
    }

    let running = statuses
        .iter()
        .filter(|status| {
            matches!(
                status,
                PodStatus::Known(KnownPodStatus::Running { started_at: _ })
            )
        })
        .count();

    Cell::from(format!("{}/{}", running, statuses.len()))
}
