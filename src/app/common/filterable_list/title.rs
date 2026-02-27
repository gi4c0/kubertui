use std::fmt::Display;

use ratatui::text::{Line, Span};

use crate::app::common::FOCUS_COLOR;

pub fn build_title<T>(titles: &[T], current: T) -> Line<'_>
where
    T: Display + PartialEq,
{
    Line::default().spans(titles.iter().enumerate().map(|(index, view)| {
        let title_text = if index == titles.len() - 1 {
            view.to_string()
        } else {
            format!("{view}   ")
        };

        let mut span = Span::from(title_text);
        if current == *view {
            span = span.style(FOCUS_COLOR);
        }

        span
    }))
}
