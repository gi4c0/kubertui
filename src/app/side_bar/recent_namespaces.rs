use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
};

#[derive(Default)]
pub struct RecentNamespacesList {
    state: ListState,
    list: Vec<String>,
}

#[derive(Default)]
pub struct RecentNamespaceResponse {
    pub is_exit: bool,
    pub selected: Option<String>,
}

impl RecentNamespacesList {
    pub fn add_to_list(&mut self, new_namespace: String) {
        if let Some(index) = self.list.iter().position(|item| item == &new_namespace) {
            self.list.remove(index);
        }

        self.list.insert(0, new_namespace);
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame) {
        let namespaces_list_items: Vec<ListItem> = self
            .list
            .iter()
            .map(|namespace| ListItem::new(namespace.as_str()))
            .collect();

        let list = List::new(namespaces_list_items)
            .block(
                Block::default()
                    .title("Recent Namespaces")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::Cyan)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_stateful_widget(list, area, &mut self.state);
    }

    fn select_next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == self.list.len() - 1 {
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
                    self.list.len() - 1
                } else {
                    i - 1
                }
            }
            None => self.list.len() - 1,
        };

        self.state.select(Some(i));
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> RecentNamespaceResponse {
        match key.code {
            KeyCode::Char('q') => {
                return RecentNamespaceResponse {
                    selected: None,
                    is_exit: true,
                };
            }
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_prev(),
            KeyCode::Enter => {
                return RecentNamespaceResponse {
                    is_exit: false,
                    selected: self.list.get(self.state.selected().unwrap_or(0)).cloned(),
                };
            }
            _ => {}
        };

        RecentNamespaceResponse::default()
    }
}
