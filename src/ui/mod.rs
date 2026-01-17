pub mod dialogs;
pub mod drawing;
pub mod handlers;
pub mod header;
pub mod toolbar;

use gtk4 as gtk;
use libadwaita as adw;

use adw::prelude::*;
use gtk::Orientation;
use std::cell::RefCell;
use std::rc::Rc;

use crate::app::{AppState, CaptureMode};

pub fn build_ui(app: &adw::Application, start_mode: Option<CaptureMode>) {
    let state = Rc::new(RefCell::new(AppState::new()));

    let header = header::create_header_bar(&state);
    let toolbar = toolbar::create_toolbar(&state);
    let crop_toolbar = toolbar::create_crop_toolbar();
    let drawing = drawing::create_drawing_area(&state);
    let text_popover = dialogs::create_text_popover(&drawing.drawing_area);

    dialogs::connect_text_popover(&state, &drawing.drawing_area, &text_popover);

    toolbar::connect_tool_buttons(
        &state,
        &toolbar,
        &toolbar.tools_box,
        &crop_toolbar.crop_tools_box,
    );

    toolbar.tools_box.set_visible(false);

    let overlay = gtk::Overlay::builder().child(&drawing.drawing_area).build();
    overlay.add_overlay(&drawing.placeholder_icon);
    overlay.add_overlay(&toolbar.tools_box);
    overlay.add_overlay(&crop_toolbar.crop_tools_box);
    overlay.add_overlay(&drawing.picked_color_label);

    let content = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .build();
    content.append(&header.header_bar);
    content.append(&overlay);

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("GNOME Snapper")
        .content(&content)
        .default_width(900)
        .default_height(600)
        .build();

    let components = handlers::UiComponents {
        window: window.clone(),
        header,
        toolbar,
        crop_toolbar,
        drawing,
        text_popover,
    };

    handlers::connect_all_handlers(&state, &components);

    window.present();

    if let Some(mode) = start_mode {
        match mode {
            CaptureMode::Selection | CaptureMode::Screen => {
                handlers::capture_screen_or_selection(
                    &state,
                    &components.window,
                    &components.header.header_bar,
                    &components.toolbar.tools_box,
                    &components.crop_toolbar.crop_tools_box,
                    &components.drawing.drawing_area,
                    &components.drawing.placeholder_icon,
                    mode,
                );
            }
            CaptureMode::Window => {
                dialogs::show_window_selector(
                    &state,
                    &components.window,
                    &components.drawing.drawing_area,
                    &components.drawing.placeholder_icon,
                    &components.toolbar.tools_box,
                );
            }
        }
    }
}
