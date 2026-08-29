use ratatui::style::Style;

pub trait ListItemTrait {
    fn as_ref(&self) -> &str;

    fn get_style(&self) -> Option<Style> {
        None
    }

    fn spinner(&self) -> Option<String> {
        None
    }
}

impl ListItemTrait for String {
    fn as_ref(&self) -> &str {
        self.as_str()
    }

    fn get_style(&self) -> Option<Style> {
        None
    }
}
