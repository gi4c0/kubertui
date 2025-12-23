use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
};

pub struct NamespacesList {
    list: Vec<String>,
    state: ListState,
}

impl Default for NamespacesList {
    fn default() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            list: vec![],
            state,
        }
    }
}

impl NamespacesList {
    pub fn draw(&mut self, area: Rect, frame: &mut Frame) {
        let namespaces_list_items: Vec<ListItem> = self
            .list
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

        frame.render_stateful_widget(list, area, &mut self.state);
    }

    pub fn update_list(&mut self, new_list: Vec<String>) {
        self.list = new_list;
    }

    pub fn select_next(&mut self) {
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

    pub fn select_prev(&mut self) {
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
}
