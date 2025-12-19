use gtk4 as gtk;
use libadwaita as adw;

use adw::prelude::*;
use gtk::{Align, DrawingArea, GestureDrag, Orientation};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

mod capture;

use capture::{capture_primary_monitor, capture_window_by_index, list_capturable_windows};

const APP_ID: &str = "org.example.ScreenshotGnome";

#[derive(Default, Clone, Copy, Debug, PartialEq)]
enum CaptureMode {
    #[default]
    Selection,
    Window,
    Screen,
}

#[derive(Default, Clone, Copy, Debug)]
struct Selection {
    start_x: f64,
    start_y: f64,
    end_x: f64,
    end_y: f64,
}

impl Selection {
    fn rectangle(&self) -> gtk::gdk::Rectangle {
        let x = self.start_x.min(self.end_x) as i32;
        let y = self.start_y.min(self.end_y) as i32;
        let w = (self.start_x - self.end_x).abs() as i32;
        let h = (self.start_y - self.end_y).abs() as i32;
        gtk::gdk::Rectangle::new(x, y, w, h)
    }
}

struct AppState {
    mode: CaptureMode,
    original_screenshot: Option<gtk::gdk_pixbuf::Pixbuf>,
    final_image: Option<gtk::gdk_pixbuf::Pixbuf>,
    selection: Option<Selection>,
    is_active: bool, // Overlay active
    monitor_x: i32,
    monitor_y: i32,
}

fn main() {
    let app = adw::Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &adw::Application) {
    let state = Rc::new(RefCell::new(AppState {
        mode: CaptureMode::Selection,
        original_screenshot: None,
        final_image: None,
        selection: None,
        is_active: false,
        monitor_x: 0,
        monitor_y: 0,
    }));

    // --- Header Bar ---
    let take_screenshot_btn = gtk::Button::builder()
        .label("Take Screenshot")
        .icon_name("camera-photo-symbolic")
        .build();
    take_screenshot_btn.add_css_class("suggested-action");

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

    // Connect mode toggles
    mode_selection.connect_toggled({
        let state = state.clone();
        move |btn| {
            if btn.is_active() {
                state.borrow_mut().mode = CaptureMode::Selection;
            }
        }
    });
    mode_window.connect_toggled({
        let state = state.clone();
        move |btn| {
            if btn.is_active() {
                state.borrow_mut().mode = CaptureMode::Window;
            }
        }
    });
    mode_screen.connect_toggled({
        let state = state.clone();
        move |btn| {
            if btn.is_active() {
                state.borrow_mut().mode = CaptureMode::Screen;
            }
        }
    });

    let title_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .build();
    title_box.append(&mode_label);
    title_box.append(&mode_box);

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

    let header_bar = adw::HeaderBar::builder().title_widget(&title_box).build();
    header_bar.pack_start(&take_screenshot_btn);
    header_bar.pack_end(&end_box);

    // --- Drawing Area ---
    let drawing_area = DrawingArea::builder().hexpand(true).vexpand(true).build();

    drawing_area.set_draw_func({
        let state = state.clone();
        move |_, cr, width, height| {
            let state = state.borrow();

            // Background
            cr.set_source_rgb(0.14, 0.14, 0.14);
            cr.paint().expect("Invalid cairo surface state");

            let image_to_draw = if state.is_active {
                state.original_screenshot.as_ref()
            } else {
                state.final_image.as_ref()
            };

            if let Some(pixbuf) = image_to_draw {
                let da_width = width as f64;
                let da_height = height as f64;
                let img_width = pixbuf.width() as f64;
                let img_height = pixbuf.height() as f64;

                // Calculate scale to fit
                let scale_x = da_width / img_width;
                let scale_y = da_height / img_height;
                let scale = scale_x.min(scale_y);

                let offset_x = if state.is_active {
                    0.0
                } else {
                    (da_width - img_width * scale) / 2.0
                };
                let offset_y = if state.is_active {
                    0.0
                } else {
                    (da_height - img_height * scale) / 2.0
                };

                cr.save().expect("Failed to save cairo context");
                cr.translate(offset_x, offset_y);
                cr.scale(scale, scale);
                cr.set_source_pixbuf(pixbuf, 0.0, 0.0);
                cr.paint().expect("Failed to paint pixbuf");
                cr.restore().expect("Failed to restore cairo context");

                // Draw Selection Overlay only if cropping
                if state.is_active && state.mode == CaptureMode::Selection {
                    if let Some(sel) = state.selection {
                        let rect = sel.rectangle();
                        let rx = rect.x() as f64;
                        let ry = rect.y() as f64;
                        let rw = rect.width() as f64;
                        let rh = rect.height() as f64;

                        // Dimming
                        cr.set_source_rgba(0.0, 0.0, 0.0, 0.5);
                        // Top
                        cr.rectangle(0.0, 0.0, da_width, ry);
                        // Bottom
                        cr.rectangle(0.0, ry + rh, da_width, da_height - (ry + rh));
                        // Left
                        cr.rectangle(0.0, ry, rx, rh);
                        // Right
                        cr.rectangle(rx + rw, ry, da_width - (rx + rw), rh);
                        cr.fill().expect("Failed to fill dimming rects");

                        // Border
                        cr.set_source_rgb(1.0, 1.0, 1.0);
                        cr.set_line_width(2.0);
                        cr.rectangle(rx, ry, rw, rh);
                        cr.stroke().expect("Failed to stroke selection border");
                    }
                }
            }
        }
    });

    // --- Main Toolbar ---
    let tools_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .halign(Align::Center)
        .valign(Align::End)
        .margin_bottom(24)
        .build();
    tools_box.add_css_class("osd");
    tools_box.add_css_class("toolbar");

    let tool_icons = [
        "input-mouse-symbolic",
        "document-edit-symbolic",
        "system-search-symbolic",
        "zoom-fit-best-symbolic",
        "crop-symbolic",
        "media-playback-stop-symbolic",
        "insert-text-symbolic",
        "color-select-symbolic",
    ];
    for icon in tool_icons {
        let btn = gtk::Button::builder().icon_name(icon).build();
        btn.add_css_class("flat");
        tools_box.append(&btn);
    }
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

    // --- Crop Confirmation Toolbar ---
    let crop_tools_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .halign(Align::Center)
        .valign(Align::End)
        .margin_bottom(24)
        .visible(false)
        .build();
    crop_tools_box.add_css_class("osd");
    crop_tools_box.add_css_class("toolbar");

    let cancel_btn = gtk::Button::builder()
        .icon_name("process-stop-symbolic")
        .tooltip_text("Cancel")
        .build();
    cancel_btn.add_css_class("destructive-action");

    let confirm_btn = gtk::Button::builder()
        .icon_name("object-select-symbolic")
        .tooltip_text("Confirm Crop")
        .build();
    confirm_btn.add_css_class("suggested-action");

    crop_tools_box.append(&cancel_btn);
    crop_tools_box.append(&confirm_btn);

    // --- Placeholder ---
    let placeholder_icon = gtk::Image::builder()
        .icon_name("image-x-generic-symbolic")
        .pixel_size(128)
        .opacity(0.2)
        .halign(Align::Center)
        .valign(Align::Center)
        .build();

    // --- Layout ---
    let overlay = gtk::Overlay::builder().child(&drawing_area).build();
    overlay.add_overlay(&placeholder_icon);
    overlay.add_overlay(&tools_box);
    overlay.add_overlay(&crop_tools_box);

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

    // --- Logic ---

    // Drag Controller
    let drag = GestureDrag::new();

    // Drag Begin
    drag.connect_drag_begin({
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        move |_, x, y| {
            let mut s = state.borrow_mut();
            if s.is_active && s.mode == CaptureMode::Selection {
                s.selection = Some(Selection {
                    start_x: x,
                    start_y: y,
                    end_x: x,
                    end_y: y,
                });
                drawing_area.queue_draw();
            }
        }
    });

    // Drag Update
    drag.connect_drag_update({
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        move |_, x, y| {
            let mut s = state.borrow_mut();
            if s.is_active && s.mode == CaptureMode::Selection {
                if let Some(sel) = &mut s.selection {
                    sel.end_x = sel.start_x + x;
                    sel.end_y = sel.start_y + y;
                    drawing_area.queue_draw();
                }
            }
        }
    });

    // Drag End (Just update selection)
    drag.connect_drag_end({
        let state = state.clone();
        let drawing_area = drawing_area.clone();

        move |_, x, y| {
            let mut s = state.borrow_mut();
            if s.is_active && s.mode == CaptureMode::Selection {
                if let Some(sel) = &mut s.selection {
                    sel.end_x = sel.start_x + x;
                    sel.end_y = sel.start_y + y;
                    drawing_area.queue_draw();
                }
            }
        }
    });
    drawing_area.add_controller(drag);

    // Confirm Crop Action
    confirm_btn.connect_clicked({
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        let window = window.clone();
        let header_bar = header_bar.clone();
        let tools_box = tools_box.clone();
        let crop_tools_box = crop_tools_box.clone();

        move |_| {
            let mut s = state.borrow_mut();
            if s.is_active && s.mode == CaptureMode::Selection {
                if let Some(sel) = s.selection {
                    let rect = sel.rectangle();

                    // Only crop if area is significant
                    if rect.width() > 10 && rect.height() > 10 {
                        if let Some(orig) = &s.original_screenshot {
                            // Assuming scale 1.0
                            let crop_x = rect.x().max(0);
                            let crop_y = rect.y().max(0);
                            let crop_w = rect.width().min(orig.width() - crop_x);
                            let crop_h = rect.height().min(orig.height() - crop_y);

                            if crop_w > 0 && crop_h > 0 {
                                let cropped = orig.new_subpixbuf(crop_x, crop_y, crop_w, crop_h);
                                s.final_image = Some(cropped);
                            }
                        }
                    }
                }

                // Exit cropping mode
                s.is_active = false;
                s.selection = None;

                // Restore UI
                window.unfullscreen();
                header_bar.set_visible(true);
                tools_box.set_visible(true);
                crop_tools_box.set_visible(false);

                drawing_area.queue_draw();
            }
        }
    });

    // Cancel Crop Action
    cancel_btn.connect_clicked({
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        let window = window.clone();
        let header_bar = header_bar.clone();
        let tools_box = tools_box.clone();
        let crop_tools_box = crop_tools_box.clone();
        let placeholder_icon = placeholder_icon.clone();

        move |_| {
            let mut s = state.borrow_mut();
            s.is_active = false;
            s.selection = None;

            window.unfullscreen();
            header_bar.set_visible(true);
            tools_box.set_visible(true);
            crop_tools_box.set_visible(false);

            if s.final_image.is_none() {
                placeholder_icon.set_visible(true);
            }

            drawing_area.queue_draw();
        }
    });

    // Take Screenshot Action
    take_screenshot_btn.connect_clicked({
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        let placeholder_icon = placeholder_icon.clone();
        let window = window.clone();
        let header_bar = header_bar.clone();
        let tools_box = tools_box.clone();
        let crop_tools_box = crop_tools_box.clone();

        move |_| {
            let mode = state.borrow().mode;

            if mode == CaptureMode::Window {
                // Open Window Selector Modal
                let window_selector = gtk::Window::builder()
                    .title("Select Window")
                    .modal(true)
                    .transient_for(&window)
                    .default_width(400)
                    .default_height(500)
                    .build();

                let list_box = gtk::ListBox::builder()
                    .selection_mode(gtk::SelectionMode::Single)
                    .css_classes(["boxed-list"])
                    .build();

                let scrolled_window = gtk::ScrolledWindow::builder()
                    .child(&list_box)
                    .vexpand(true)
                    .build();

                let vbox = gtk::Box::builder()
                    .orientation(Orientation::Vertical)
                    .spacing(12)
                    .margin_top(12)
                    .margin_bottom(12)
                    .margin_start(12)
                    .margin_end(12)
                    .build();

                vbox.append(&gtk::Label::new(Some("Select a window to capture:")));
                vbox.append(&scrolled_window);
                window_selector.set_child(Some(&vbox));

                // Populate List using capture module
                if let Ok(windows) = list_capturable_windows() {
                    for win_info in &windows {
                        let row = gtk::Box::builder()
                            .orientation(Orientation::Horizontal)
                            .spacing(12)
                            .build();

                        // Icon - use icon_name_hint from WindowInfo
                        let icon = gtk::Image::builder()
                            .icon_name(win_info.icon_name_hint().to_lowercase())
                            .pixel_size(32)
                            .build();

                        let label = gtk::Label::builder()
                            .label(&win_info.display_label())
                            .halign(Align::Start)
                            .ellipsize(gtk::pango::EllipsizeMode::End)
                            .build();

                        row.append(&icon);
                        row.append(&label);

                        list_box.append(&row);
                    }
                }

                // Handle Selection
                let state_clone = state.clone();
                let drawing_area_clone = drawing_area.clone();
                let placeholder_icon_clone = placeholder_icon.clone();
                let window_selector_clone = window_selector.clone();

                list_box.connect_row_activated(move |_lb, row| {
                    let idx = row.index();
                    if idx >= 0 {
                        // Use capture module to capture window by index
                        if let Ok(result) = capture_window_by_index(idx as usize) {
                            let mut s = state_clone.borrow_mut();
                            s.final_image = Some(result.pixbuf);
                            s.is_active = false;

                            placeholder_icon_clone.set_visible(false);
                            drawing_area_clone.queue_draw();
                        }
                    }
                    window_selector_clone.close();
                });

                window_selector.present();
            } else {
                // Screen or Selection Mode

                // Hide window
                window.set_visible(false);

                let context = gtk::glib::MainContext::default();
                while context.pending() {
                    context.iteration(false);
                }
                std::thread::sleep(Duration::from_millis(200));

                // Capture using the capture module
                match capture_primary_monitor() {
                    Ok(result) => {
                        let mut s = state.borrow_mut();
                        s.monitor_x = result.monitor_info.x;
                        s.monitor_y = result.monitor_info.y;

                        if mode == CaptureMode::Screen {
                            s.final_image = Some(result.pixbuf);
                            s.is_active = false;
                            window.set_visible(true);
                            placeholder_icon.set_visible(false);
                            drawing_area.queue_draw();
                        } else {
                            // Selection Mode
                            s.original_screenshot = Some(result.pixbuf);
                            s.is_active = true;
                            s.selection = None;

                            placeholder_icon.set_visible(false);
                            header_bar.set_visible(false);
                            tools_box.set_visible(false);
                            crop_tools_box.set_visible(true);

                            window.set_visible(true);
                            window.fullscreen();
                            drawing_area.queue_draw();
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to capture screen: {}", e);
                        window.set_visible(true);
                    }
                }
            }
        }
    });

    window.present();
}
