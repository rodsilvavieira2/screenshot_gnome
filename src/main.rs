use gtk4 as gtk;
use libadwaita as adw;

use adw::prelude::*;
use gtk::{Align, DrawingArea, GestureDrag, Orientation};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use xcap::Monitor;

const APP_ID: &str = "org.example.ScreenshotGnome";

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
    original_screenshot: Option<gtk::gdk_pixbuf::Pixbuf>,
    final_image: Option<gtk::gdk_pixbuf::Pixbuf>,
    selection: Option<Selection>,
    is_cropping: bool,
}

fn main() {
    let app = adw::Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &adw::Application) {
    let state = Rc::new(RefCell::new(AppState {
        original_screenshot: None,
        final_image: None,
        selection: None,
        is_cropping: false,
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

            let image_to_draw = if state.is_cropping {
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

                let offset_x = if state.is_cropping {
                    0.0
                } else {
                    (da_width - img_width * scale) / 2.0
                };
                let offset_y = if state.is_cropping {
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
                if state.is_cropping {
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

    // --- Toolbar ---
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
            if s.is_cropping {
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
            if s.is_cropping {
                if let Some(sel) = &mut s.selection {
                    sel.end_x = sel.start_x + x;
                    sel.end_y = sel.start_y + y;
                    drawing_area.queue_draw();
                }
            }
        }
    });

    // Drag End (Perform Crop)
    drag.connect_drag_end({
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        let window = window.clone();
        let header_bar = header_bar.clone();
        let tools_box = tools_box.clone();

        move |_, x, y| {
            let mut s = state.borrow_mut();
            if s.is_cropping {
                if let Some(sel) = &mut s.selection {
                    sel.end_x = sel.start_x + x;
                    sel.end_y = sel.start_y + y;

                    // Calculate crop rectangle
                    let rect = sel.rectangle();

                    // Only crop if area is significant
                    if rect.width() > 10 && rect.height() > 10 {
                        if let Some(orig) = &s.original_screenshot {
                            // Calculate scale used in draw
                            let da_width = drawing_area.width() as f64;
                            let da_height = drawing_area.height() as f64;
                            let img_width = orig.width() as f64;
                            let img_height = orig.height() as f64;

                            let scale_x = da_width / img_width;
                            let scale_y = da_height / img_height;
                            let scale = scale_x.min(scale_y);

                            // Transform widget coords to image coords
                            let crop_x = (rect.x() as f64 / scale) as i32;
                            let crop_y = (rect.y() as f64 / scale) as i32;
                            let crop_w = (rect.width() as f64 / scale) as i32;
                            let crop_h = (rect.height() as f64 / scale) as i32;

                            // Clamp to image bounds
                            let crop_x = crop_x.max(0).min(orig.width() - 1);
                            let crop_y = crop_y.max(0).min(orig.height() - 1);
                            let crop_w = crop_w.min(orig.width() - crop_x);
                            let crop_h = crop_h.min(orig.height() - crop_y);

                            if crop_w > 0 && crop_h > 0 {
                                let cropped = orig.new_subpixbuf(crop_x, crop_y, crop_w, crop_h);
                                s.final_image = Some(cropped);
                            }
                        }

                        // Exit cropping mode
                        s.is_cropping = false;
                        s.selection = None;

                        // Restore UI
                        window.unfullscreen();
                        header_bar.set_visible(true);
                        tools_box.set_visible(true);

                        drawing_area.queue_draw();
                    } else {
                        // Selection too small, just clear it
                        s.selection = None;
                        drawing_area.queue_draw();
                    }
                }
            }
        }
    });
    drawing_area.add_controller(drag);

    // Take Screenshot Action
    take_screenshot_btn.connect_clicked({
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        let placeholder_icon = placeholder_icon.clone();
        let window = window.clone();
        let header_bar = header_bar.clone();
        let tools_box = tools_box.clone();

        move |_| {
            // Hide window
            window.set_visible(false);

            let context = gtk::glib::MainContext::default();
            while context.pending() {
                context.iteration(false);
            }
            std::thread::sleep(Duration::from_millis(200));

            // Capture
            let monitors = Monitor::all().unwrap_or_default();
            if let Some(monitor) = monitors.first() {
                if let Ok(image) = monitor.capture_image() {
                    let width = image.width() as i32;
                    let height = image.height() as i32;
                    let stride = width * 4;
                    let pixels = image.into_raw();
                    let bytes = gtk::glib::Bytes::from(&pixels);

                    let pixbuf = gtk::gdk_pixbuf::Pixbuf::from_bytes(
                        &bytes,
                        gtk::gdk_pixbuf::Colorspace::Rgb,
                        true,
                        8,
                        width,
                        height,
                        stride,
                    );

                    let mut s = state.borrow_mut();
                    s.original_screenshot = Some(pixbuf);
                    s.is_cropping = true;
                    s.selection = None;
                }
            }

            // Prepare UI for cropping
            placeholder_icon.set_visible(false);
            header_bar.set_visible(false);
            tools_box.set_visible(false);

            window.set_visible(true);
            window.fullscreen();
            drawing_area.queue_draw();
        }
    });

    window.present();
}
