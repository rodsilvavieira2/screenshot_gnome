use gtk4 as gtk;
use libadwaita as adw;

use adw::prelude::*;

use gtk::{Align, Orientation};

const APP_ID: &str = "org.example.ScreenshotGnome";

fn main() {
    let app = adw::Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &adw::Application) {
    // --- Header Bar Content ---

    // Start: Take Screenshot Button
    let take_screenshot_btn = gtk::Button::builder()
        .label("Take Screenshot")
        .icon_name("camera-photo-symbolic")
        .build();
    take_screenshot_btn.add_css_class("suggested-action");

    // Title: Mode Selector
    let mode_label = gtk::Label::new(Some("Mode:"));
    mode_label.add_css_class("dim-label");

    let mode_selection = gtk::ToggleButton::builder()
        .label("Selection")
        .active(true)
        .build();
    let mode_window = gtk::ToggleButton::builder()
        .label("Window")
        .group(&mode_selection)
        .build();
    let mode_screen = gtk::ToggleButton::builder()
        .label("Screen")
        .group(&mode_selection)
        .build();

    let mode_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .build();
    mode_box.add_css_class("linked");
    mode_box.append(&mode_selection);
    mode_box.append(&mode_window);
    mode_box.append(&mode_screen);

    let title_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .build();
    title_box.append(&mode_label);
    title_box.append(&mode_box);

    // End: Delay Controls & Menu
    let delay_label = gtk::Label::new(Some("Delay:"));
    delay_label.add_css_class("dim-label");

    let delay_value = gtk::Label::builder().label("0").width_chars(2).build();

    let delay_minus = gtk::Button::builder()
        .icon_name("list-remove-symbolic")
        .build();
    let delay_plus = gtk::Button::builder()
        .icon_name("list-add-symbolic")
        .build();

    let delay_controls = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .build();
    delay_controls.add_css_class("linked");
    delay_controls.append(&delay_minus);
    delay_controls.append(&delay_plus);

    let menu_btn = gtk::MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .build();

    let end_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .build();

    end_box.append(&delay_label);
    end_box.append(&delay_value);
    end_box.append(&delay_controls);
    end_box.append(&menu_btn);

    // Assemble Header Bar
    let header_bar = adw::HeaderBar::builder().title_widget(&title_box).build();

    header_bar.pack_start(&take_screenshot_btn);
    header_bar.pack_end(&end_box);

    // --- Main Content Area ---

    // Placeholder for the screenshot image/canvas
    // We use a Frame with a dark background to simulate the capture area
    let canvas_area = gtk::Frame::builder().hexpand(true).vexpand(true).build();

    let placeholder_icon = gtk::Image::builder()
        .icon_name("image-x-generic-symbolic")
        .pixel_size(128)
        .opacity(0.2)
        .halign(Align::Center)
        .valign(Align::Center)
        .build();
    canvas_area.set_child(Some(&placeholder_icon));

    // Add a CSS provider to style the canvas dark
    let css_provider = gtk::CssProvider::new();
    css_provider.load_from_string("frame { background-color: #242424; }");
    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("Could not connect to a display."),
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // --- Bottom Floating Toolbar ---

    let tools_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .halign(Align::Center)
        .valign(Align::End)
        .margin_bottom(24)
        .build();
    // 'osd' gives the dark rounded overlay look
    tools_box.add_css_class("osd");
    tools_box.add_css_class("toolbar");

    // Tool buttons
    let tool_icons = [
        "input-mouse-symbolic",         // Pointer
        "document-edit-symbolic",       // Pencil
        "system-search-symbolic",       // Magnifier
        "zoom-fit-best-symbolic",       // Resize
        "crop-symbolic",                // Crop
        "media-playback-stop-symbolic", // Square shape
        "insert-text-symbolic",         // Text
        "color-select-symbolic",        // Color
    ];

    for icon in tool_icons {
        let btn = gtk::Button::builder().icon_name(icon).build();
        btn.add_css_class("flat");
        tools_box.append(&btn);
    }

    // Spacer
    // In a packed box, hexpand spacer pushes things apart, but here we want a gap before Save.
    // Since the box is halign=center, it won't expand to fill width.
    // We just add a margin to the save button or a separator.
    let separator = gtk::Separator::builder()
        .orientation(Orientation::Vertical)
        .margin_start(6)
        .margin_end(6)
        .build();
    separator.add_css_class("spacer");
    tools_box.append(&separator);

    let save_btn = gtk::Button::builder()
        .label("Save")
        .icon_name("document-save-symbolic")
        .build();
    save_btn.add_css_class("suggested-action");
    tools_box.append(&save_btn);

    // Overlay to stack canvas and toolbar
    let overlay = gtk::Overlay::builder().child(&canvas_area).build();
    overlay.add_overlay(&tools_box);

    // --- Window Setup ---

    let content = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .build();
    content.append(&header_bar);
    content.append(&overlay);

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("GNOME Snapper")
        .content(&content)
        .default_width(900)
        .default_height(600)
        .build();

    window.present();
}
