mod chromedriver;
mod application;
mod filemanager;
mod downloader;

use crate::application::Application;

#[tokio::main]
async fn main() {
    let mut terminal = ratatui::init();
    let mut application: Application = Application::new();
    application.run(&mut terminal).await;
    ratatui::restore();
}
