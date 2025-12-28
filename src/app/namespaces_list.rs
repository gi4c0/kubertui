use crate::app::{
    cache::{NamespacesListCache, StateCache},
    common::{FOCUS_COLOR, build_block, get_highlight_style},
    events::{AppEvent, EventSender},
};
use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
};

#[derive(Debug, Clone)]
pub struct NamespacesList {
    original_list: Vec<String>,
    filtered_list: Vec<String>,
    state: ListState,
    filter: String,
    is_filter_mod: bool,
    event_sender: EventSender,
}

impl From<NamespacesList> for NamespacesListCache {
    fn from(value: NamespacesList) -> Self {
        Self {
            filter: value.filter,
            is_filter_mod: value.is_filter_mod,
            filtered_list: value.filtered_list,
            original_list: value.original_list,
            state: StateCache {
                selected: value.state.selected(),
            },
        }
    }
}

impl NamespacesList {
    pub fn new(event_sender: EventSender) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            original_list: vec![],
            filtered_list: vec![],
            state,
            filter: String::new(),
            is_filter_mod: false,
            event_sender,
        }
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame) {
        self.filtered_list = self
            .original_list
            .iter()
            .filter(|item| {
                if self.filter.is_empty() {
                    return true;
                }

                item.contains(&self.filter)
            })
            .map(|item| item.to_owned())
            .collect();

        let namespaces_list_items: Vec<ListItem> = self
            .filtered_list
            .iter()
            .map(|namespace| ListItem::new(namespace.as_str()))
            .collect();

        let list = List::new(namespaces_list_items)
            .block(build_block("Select Namespace"))
            .highlight_style(get_highlight_style());

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
            frame.render_stateful_widget(list, layouts[1], &mut self.state);
            return;
        }
        frame.render_stateful_widget(list, area, &mut self.state);
    }

    pub fn update_list(&mut self, new_list: Vec<String>) {
        self.original_list = new_list;
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

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        if self.is_filter_mod && key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Enter => {
                    self.is_filter_mod = false;
                    self.state.select(Some(0));
                }
                KeyCode::Esc => {
                    self.filter.clear();
                    self.is_filter_mod = false;
                }
                KeyCode::Backspace => {
                    self.filter.pop();
                }
                KeyCode::Char(ch) => {
                    self.filter.push(ch);
                }
                _ => {}
            };

            return;
        }

        match key.code {
            KeyCode::Char('q') => {
                let _ = self.event_sender.send(AppEvent::Quit);
            }
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_prev(),
            KeyCode::Char('/') => {
                self.is_filter_mod = true;
            }
            KeyCode::Enter => {
                let _ = self.event_sender.send(AppEvent::SelectNamespace(
                    self.filtered_list[self.state.selected().unwrap_or(0)].clone(),
                ));
            }
            _ => {}
        };
    }
}
