mod port_forward_popup;

use crossterm::event::KeyCode;

use ratatui::{
    Frame,
    crossterm::event::KeyEvent,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Cell, Paragraph, Row, Table, TableState},
};

use crate::{
    app::{
        cache::{PodsListCache, StateCache},
        common::{FOCUS_COLOR, build_block, get_highlight_style},
        events::{AppEvent, EventSender},
        pods_list::port_forward_popup::{PortForwardPopup, PortForwardPopupAction},
    },
    error::AppResult,
    kubectl::pods::{KnownPodStatus, Pod, PodStatus, get_pods_list},
};

#[derive(Debug, Clone)]
pub struct PodsList {
    original_list: Vec<Pod>,
    filtered_list: Vec<Pod>,
    event_sender: EventSender,
    state: TableState,
    filter: String,
    is_filter_mod: bool,
    longest_name: u16,
    port_forward_popup: Option<PortForwardPopup>,
    namespace: String,
}

impl From<PodsList> for PodsListCache {
    fn from(value: PodsList) -> Self {
        Self {
            filter: value.filter,
            filtered_list: value.filtered_list,
            is_filter_mod: value.is_filter_mod,
            original_list: value.original_list,
            longest_name: value.longest_name,
            namespace: value.namespace,
            state: StateCache {
                selected: value.state.selected(),
            },
            port_forward_popup: match value.port_forward_popup {
                Some(port_forward_popup) => {
                    let i = (&port_forward_popup).into();
                    Some(i)
                }
                None => None,
            },
        }
    }
}

impl PodsList {
    pub async fn new(event_sender: EventSender, namespace: String) -> AppResult<Self> {
        let pods = get_pods_list(namespace.as_str()).await?;

        let mut state = TableState::new();
        state.select(Some(0));

        let longest_name = pods
            .iter()
            .max_by_key(|p| p.name.len())
            .map(|p| p.name.len())
            .unwrap_or(10) as u16;

        Ok(Self {
            filtered_list: pods.clone(),
            namespace,
            longest_name,
            original_list: pods,
            event_sender,
            state,
            filter: String::new(),
            is_filter_mod: false,
            port_forward_popup: None,
        })
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame) {
        self.draw_list(area, frame);
    }

    fn draw_list(&mut self, area: Rect, frame: &mut Frame) {
        let header = ["Name", "Containers"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>();

        self.filtered_list = self
            .original_list
            .iter()
            .filter(|item| {
                if self.filter.is_empty() {
                    return true;
                }

                item.name.contains(&self.filter)
            })
            .map(|item| item.to_owned())
            .collect();

        let rows: Vec<Row> = self
            .filtered_list
            .iter()
            .map(|item| {
                Row::new([
                    item.name.as_str().into(),
                    get_status(&item.container_statuses),
                ])
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Length(self.longest_name + 3),
                Constraint::Min(5),
            ],
        )
        .header(header)
        .block(build_block("Select pod"))
        .row_highlight_style(get_highlight_style());

        if self.is_filter_mod || !self.filter.is_empty() {
            let layouts = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Length(3), Constraint::Min(1)])
                .split(area);

            let mut block = build_block("Filter");

            if self.is_filter_mod {
                block = block.border_style(FOCUS_COLOR);
            }

            let filter_widget = Paragraph::new(self.filter.as_str()).block(block);

            frame.render_widget(filter_widget, layouts[0]);
            frame.render_stateful_widget(table, layouts[1], &mut self.state);
            return;
        }

        frame.render_stateful_widget(table, area, &mut self.state);

        if let Some(port_forward_popup) = &mut self.port_forward_popup {
            port_forward_popup.draw(frame);
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        if let Some(port_forward_popup) = &mut self.port_forward_popup
            && let Some(port_forward_popup_action) = port_forward_popup.handle_key_event(key)
        {
            return match port_forward_popup_action {
                PortForwardPopupAction::PortForward {
                    local_port,
                    app_port,
                } => {
                    let pod = self.filtered_list[self.state.selected().unwrap_or(0)].clone();

                    let _ = self.event_sender.send(AppEvent::PortForward {
                        pod_name: pod.name,
                        local_port,
                        app_port,
                        namespace: self.namespace.clone(),
                    });
                }

                PortForwardPopupAction::Quit => {
                    self.port_forward_popup = None;
                }
            };
        }

        if self.is_filter_mod {
            return match key.code {
                KeyCode::Enter => {
                    self.is_filter_mod = false;
                    self.state.select(Some(0));
                }
                KeyCode::Esc => {
                    self.filter.clear();
                    self.is_filter_mod = false;
                    self.state.select(Some(0));
                }
                KeyCode::Backspace => {
                    self.filter.pop();
                }
                KeyCode::Char(ch) => {
                    self.filter.push(ch);
                }
                _ => {}
            };
        }

        match key.code {
            KeyCode::Char('q') => {
                let _ = self.event_sender.send(AppEvent::Quit);
            }
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_prev(),
            KeyCode::Char('/') => self.is_filter_mod = true,
            KeyCode::Char('p') => {
                let pod_containers = self.filtered_list[self.state.selected().unwrap_or(0)]
                    .containers
                    .clone();

                self.port_forward_popup = Some(PortForwardPopup::new(pod_containers));
            }
            _ => {}
        }
    }

    fn select_next(&mut self) {
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

fn get_status(statuses: &[PodStatus]) -> Cell<'_> {
    if statuses.len() <= 5 {
        let statuses: Vec<String> = statuses
            .iter()
            .map(|status| match status {
                PodStatus::Unknown(status) => {
                    println!("{}", status);
                    "❓".into()
                }
                PodStatus::Known(known_status) => match known_status {
                    KnownPodStatus::Running { started_at: _ } => "💚".into(),
                    KnownPodStatus::Terminated {
                        container_id: _,
                        exit_code: _,
                        finished_at: _,
                        reason: _,
                        started_at: _,
                    } => "💔".into(),
                    KnownPodStatus::Waiting {
                        reason: _,
                        message: _,
                    } => "💤".into(),
                },
            })
            .collect();

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
