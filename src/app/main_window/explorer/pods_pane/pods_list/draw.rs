use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Cell, Paragraph, Row, Table},
};

use crate::app::{
    common::{build_block, get_highlight_style},
    main_window::explorer::pods_pane::pods_list::{PodsList, utils::get_status},
};

impl PodsList {
    pub fn draw(&mut self, area: Rect, frame: &mut Frame) {
        let header = ["Name", "Containers"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>();

        let rows: Vec<Row> = self
            .filtered_list
            .iter()
            .map(|index| {
                let item = &self.original_list[*index];

                let maybe_spinner = item
                    .spinner
                    .as_ref()
                    .and_then(|spinner| spinner.get_spin_state())
                    .unwrap_or(" ");

                let pod_name = format!("{maybe_spinner} {}", item.pod.name.as_str());

                Row::new([
                    pod_name.into(),
                    get_status(&item.pod.container_statuses, &item.pod.reason),
                ])
            })
            .collect();

        let block = build_block("Select pod", false);

        let table = Table::new(
            rows,
            [
                Constraint::Length(self.longest_name + 3),
                Constraint::Min(5),
            ],
        )
        .header(header)
        .block(block)
        .row_highlight_style(get_highlight_style());

        if self.filter.is_visible() {
            let layouts = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Length(3), Constraint::Min(1)])
                .split(area);

            let block = build_block("Filter", self.filter.is_active());
            let filter_widget = Paragraph::new(self.filter.text()).block(block);

            frame.render_widget(filter_widget, layouts[0]);
            frame.render_stateful_widget(table, layouts[1], &mut self.state);
        } else {
            frame.render_stateful_widget(table, area, &mut self.state);
        }

        if let Some(pod_menu_popup) = self.pod_menu_popup.as_mut() {
            pod_menu_popup.draw(area, frame);
        }
    }
}
