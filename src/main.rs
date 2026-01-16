use libadwaita as adw;

use adw::prelude::*;

mod app;
mod capture;
mod editor;
mod ui;

const APP_ID: &str = "org.example.ScreenshotGnome";

fn main() {
    let app = adw::Application::builder().application_id(APP_ID).build();

    app.connect_activate(ui::build_ui);
    app.run();
}