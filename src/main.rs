mod app;
mod error;
mod files;
mod kubectl;

use crate::{app::App, error::AppResult};

#[tokio::main]
async fn main() -> AppResult<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal).await;
    ratatui::restore();
    app_result
}
