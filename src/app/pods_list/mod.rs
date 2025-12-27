mod port_forward_popup;

use crossterm::event::KeyCode;

use ratatui::{
    Frame,
    crossterm::event::KeyEvent,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, TableState},
};

use crate::{
    app::{
        FOCUS_COLOR, centered_rect,
        events::{AppEvent, EventSender},
        pods_list::port_forward_popup::{PortForwardPopup, PortForwardPopupAction},
    },
    error::AppResult,
    kubectl::pods::{KnownPodStatus, Pod, PodStatus, get_pods_list},
};

pub struct PodsList {
    original_list: Vec<Pod>,
    filtered_list: Vec<Pod>,
    event_sender: EventSender,
    state: TableState,
    filter: String,
    is_filter_mod: bool,
    longest_name: u16,
    port_forward_popup: Option<PortForwardPopup>,
}

impl PodsList {
    pub async fn new(event_sender: EventSender, namespace: String) -> AppResult<Self> {
        let pods = get_pods_list(namespace).await?;

        let mut state = TableState::new();
        state.select(Some(0));

        let longest_name = pods
            .iter()
            .max_by_key(|p| p.name.len())
            .map(|p| p.name.len())
            .unwrap_or(10) as u16;

        Ok(Self {
            filtered_list: pods.clone(),
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
        .block(
            Block::default()
                .title("Select Pod")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .row_highlight_style(
            Style::default()
                .bg(FOCUS_COLOR)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

        if self.is_filter_mod || !self.filter.is_empty() {
            let layouts = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Length(3), Constraint::Min(1)])
                .split(area);

            let mut block = Block::default()
                .borders(Borders::ALL)
                .title("Filter")
                .border_type(BorderType::Rounded);

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
