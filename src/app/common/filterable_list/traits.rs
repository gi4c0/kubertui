use ratatui::style::Style;

pub trait ListItemTrait {
    fn get_style(&self) -> Option<Style> {
        None
    }

    fn spinner(&self) -> Option<String> {
        None
    }
}

impl ListItemTrait for String {
    fn get_style(&self) -> Option<Style> {
        None
    }
}
