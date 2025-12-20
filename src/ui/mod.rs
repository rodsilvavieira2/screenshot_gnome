//! UI module
//!
//! This module contains all the UI components and handlers for the screenshot application.
//! It provides a clean separation of concerns with different submodules for:
//!
//! - `header`: Header bar with capture mode selection and delay controls
//! - `toolbar`: Main editing toolbar and crop confirmation toolbar
//! - `drawing`: Drawing area for displaying and editing screenshots
//! - `dialogs`: Dialogs and popovers (text input, window selector)
//! - `handlers`: Event handler connections

pub mod dialogs;
pub mod drawing;
pub mod handlers;
pub mod header;
pub mod toolbar;

// Re-export commonly used types for external use
#[allow(unused_imports)]
pub use dialogs::{
    TextPopoverComponents, connect_text_popover, create_text_popover, show_window_selector,
};
#[allow(unused_imports)]
pub use drawing::{DrawingComponents, create_drawing_area};
#[allow(unused_imports)]
pub use handlers::{UiComponents, connect_all_handlers};
#[allow(unused_imports)]
pub use header::{HeaderComponents, create_header_bar};
#[allow(unused_imports)]
pub use toolbar::{
    CropToolbarComponents, ToolbarComponents, connect_tool_buttons, create_crop_toolbar,
    create_toolbar,
};

use gtk4 as gtk;
use libadwaita as adw;

use adw::prelude::*;
use gtk::Orientation;
use std::cell::RefCell;
use std::rc::Rc;

use crate::app::AppState;

/// Build the complete UI and connect all handlers
///
/// This is the main entry point for setting up the application UI.
/// It creates all components, assembles them into a window, and connects
/// all event handlers.
pub fn build_ui(app: &adw::Application) {
    // Create application state
    let state = Rc::new(RefCell::new(AppState::new()));

    // Create UI components
    let header = header::create_header_bar(&state);
    let toolbar = toolbar::create_toolbar(&state);
    let crop_toolbar = toolbar::create_crop_toolbar();
    let drawing = drawing::create_drawing_area(&state);
    let text_popover = dialogs::create_text_popover(&drawing.drawing_area);

    // Connect text popover handlers
    dialogs::connect_text_popover(&state, &drawing.drawing_area, &text_popover);

    // Connect tool button handlers
    toolbar::connect_tool_buttons(
        &state,
        &toolbar,
        &toolbar.tools_box,
        &crop_toolbar.crop_tools_box,
    );

    // Create the overlay with all components
    let overlay = gtk::Overlay::builder().child(&drawing.drawing_area).build();
    overlay.add_overlay(&drawing.placeholder_icon);
    overlay.add_overlay(&toolbar.tools_box);
    overlay.add_overlay(&crop_toolbar.crop_tools_box);
    overlay.add_overlay(&drawing.picked_color_label);

    // Create the main content box
    let content = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .build();
    content.append(&header.header_bar);
    content.append(&overlay);

    // Create the main window
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("GNOME Snapper")
        .content(&content)
        .default_width(900)
        .default_height(600)
        .build();

    // Create the UiComponents container for handlers
    let components = handlers::UiComponents {
        window: window.clone(),
        header,
        toolbar,
        crop_toolbar,
        drawing,
        text_popover,
    };

    // Connect all event handlers
    handlers::connect_all_handlers(&state, &components);

    // Present the window
    window.present();
}
