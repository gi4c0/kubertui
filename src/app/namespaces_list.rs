use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
};

pub struct NamespacesList {
    list: Vec<String>,
    filtered_list: Vec<String>,
    state: ListState,
    filter: String,
    is_filter_mod: bool,
}

impl Default for NamespacesList {
    fn default() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            list: vec![],
            filtered_list: vec![],
            state,
            filter: String::new(),
            is_filter_mod: false,
        }
    }
}

impl NamespacesList {
    pub fn draw(&mut self, area: Rect, frame: &mut Frame) {
        self.filtered_list = self
            .list
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
            .block(
                Block::default()
                    .title("Select Namespace")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::Cyan)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            );

        if self.is_filter_mod || !self.filter.is_empty() {
            let layouts = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Length(3), Constraint::Min(1)])
                .split(area);

            let filter_widget = Paragraph::new(self.filter.as_str()).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Filter")
                    .border_type(BorderType::Rounded),
            );

            frame.render_widget(filter_widget, layouts[0]);
            frame.render_stateful_widget(list, layouts[1], &mut self.state);
            return;
        }
        frame.render_stateful_widget(list, area, &mut self.state);
    }

    pub fn update_list(&mut self, new_list: Vec<String>) {
        self.list = new_list;
    }

    pub fn select_next(&mut self) {
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

    pub fn select_prev(&mut self) {
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

    pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        if self.is_filter_mod && key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Enter => {
                    self.is_filter_mod = false;
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
            }
            return false;
        }

        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_prev(),
            KeyCode::Char('/') => {
                self.is_filter_mod = true;
            }
            _ => {}
        };

        false
    }
}
