use crossterm::event::{KeyCode, KeyEvent};

/// Text filter shared by list widgets: owns the filter text and whether the
/// user is currently typing into it.
#[derive(Debug, Clone, Default)]
pub struct Filter {
    text: String,
    active: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FilterEvent {
    /// Text changed: caller must recompute filtered indices.
    Changed,
    /// Left filter mode. `changed` is true when the text was cleared (Esc).
    Closed {
        changed: bool,
    },
    Ignored,
}

impl Filter {
    pub fn from_parts(text: String, active: bool) -> Self {
        Self { text, active }
    }

    pub fn into_parts(self) -> (String, bool) {
        (self.text, self.active)
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    /// True when the filter widget should be rendered.
    pub fn is_visible(&self) -> bool {
        self.active || !self.text.is_empty()
    }

    pub fn activate(&mut self) {
        self.active = true;
    }

    pub fn clear(&mut self) {
        self.text.clear();
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> FilterEvent {
        match key.code {
            KeyCode::Enter => {
                self.active = false;
                FilterEvent::Closed { changed: false }
            }
            KeyCode::Esc => {
                self.text.clear();
                self.active = false;
                FilterEvent::Closed { changed: true }
            }
            KeyCode::Backspace => {
                self.text.pop();
                FilterEvent::Changed
            }
            KeyCode::Char(ch) => {
                self.text.push(ch);
                FilterEvent::Changed
            }
            _ => FilterEvent::Ignored,
        }
    }

    /// Indices of `names` matching the filter. Empty/whitespace filter matches all.
    pub fn apply<'a>(&self, names: impl Iterator<Item = &'a str>) -> Vec<usize> {
        let needle = self.text.trim();

        names
            .enumerate()
            .filter(|(_, name)| needle.is_empty() || name.contains(needle))
            .map(|(index, _)| index)
            .collect()
    }
}
