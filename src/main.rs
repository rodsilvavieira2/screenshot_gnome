use libadwaita as adw;

use adw::prelude::*;

use crate::app::CaptureMode;

mod app;
mod capture;
mod editor;
mod ui;

const APP_ID: &str = "org.example.ScreenshotGnome";

fn main() {
    env_logger::init();
    
    let bytes = gtk4::glib::Bytes::from_static(include_bytes!("resources.gresource"));
    let resource = gtk4::gio::Resource::from_data(&bytes).unwrap();
    gtk4::gio::resources_register(&resource);

    let args: Vec<String> = std::env::args().collect();

    let start_mode =
        if args.contains(&"--selection".to_string()) || args.contains(&"-s".to_string()) {
            Some(CaptureMode::Selection)
        } else if args.contains(&"--screen".to_string()) {
            Some(CaptureMode::Screen)
        } else if args.contains(&"--window".to_string()) || args.contains(&"-w".to_string()) {
            Some(CaptureMode::Window)
        } else {
            None
        };

    let app = adw::Application::builder().application_id(APP_ID).build();

    app.connect_startup(|_| {
        if let Some(display) = gtk4::gdk::Display::default() {
            let icon_theme = gtk4::IconTheme::for_display(&display);
            icon_theme.add_resource_path("/org/example/ScreenshotGnome/icons");
        }
    });

    app.connect_activate(move |app| {
        ui::build_ui(app, start_mode);
    });

    let gtk_args: Vec<String> = args
        .into_iter()
        .filter(|a| {
            !matches!(
                a.as_str(),
                "--selection" | "-s" | "--screen" | "--window" | "-w"
            )
        })
        .collect();

    app.run_with_args(&gtk_args);
}
