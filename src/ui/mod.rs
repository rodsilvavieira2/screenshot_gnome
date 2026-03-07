pub mod dialogs;
pub mod drawing;
pub mod handlers;
pub mod header;
pub mod shortcuts;
pub mod toolbar;

use gtk4 as gtk;
use libadwaita as adw;

use adw::prelude::*;
use gtk::Orientation;
use log::info;
use std::cell::RefCell;
use std::rc::Rc;

use crate::app::{AppState, CaptureMode};

fn load_custom_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_string("
        .custom-toolbar {
            background-color: @window_bg_color;
            border: 1px solid @borders;
            border-radius: 12px;
            padding: 6px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.15);
        }
    ");
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

pub fn build_ui(app: &adw::Application, start_mode: Option<CaptureMode>) {
    info!("Building UI...");
    load_custom_css();
    let state = Rc::new(RefCell::new(AppState::new()));

    let header = header::create_header_bar(&state);
    let toolbar = toolbar::create_toolbar(&state);
    let crop_toolbar = toolbar::create_crop_toolbar();
    let selection_toolbar = toolbar::create_selection_toolbar();
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
    overlay.add_overlay(&selection_toolbar.selection_tools_box);
    overlay.add_overlay(&drawing.picked_color_label);

    let content = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .build();
    content.append(&header.header_bar);
    content.append(&overlay);

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Screenshot Tool")
        .content(&content)
        .default_width(900)
        .default_height(600)
        .build();

    let components = handlers::UiComponents {
        window: window.clone(),
        header,
        toolbar,
        crop_toolbar,
        selection_toolbar,
        drawing,
        text_popover,
    };

    handlers::connect_all_handlers(&state, &components);

    info!("Presenting main window");
    window.present();

    if let Some(mode) = start_mode {
        info!("Starting with mode: {:?}", mode);
        match mode {
            CaptureMode::Selection | CaptureMode::Screen => {
                handlers::capture_screen_or_selection(
                    &state,
                    &components,
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
