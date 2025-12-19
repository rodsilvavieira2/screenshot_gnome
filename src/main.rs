use gtk4 as gtk;
use libadwaita as adw;

use adw::prelude::*;
use gtk::{Align, DrawingArea, GestureClick, GestureDrag, Orientation};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

mod capture;
mod editor;

use capture::{capture_primary_monitor, capture_window_by_index, list_capturable_windows};
use editor::{
    Annotation, ClipboardManager, EditorState, EditorTool, FreeDrawAnnotation, RectangleAnnotation,
    pick_color_from_pixbuf,
};

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
    is_active: bool, // Overlay active (for selection mode capture)
    monitor_x: i32,
    monitor_y: i32,
    // Editor state
    editor: EditorState,
    is_crop_mode: bool, // Separate crop mode from capture selection
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
        editor: EditorState::new(),
        is_crop_mode: false,
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
            let mut state = state.borrow_mut();

            // Background
            cr.set_source_rgb(0.14, 0.14, 0.14);
            cr.paint().expect("Invalid cairo surface state");

            // Clone the pixbuf to avoid borrow issues
            let pixbuf_opt = if state.is_active {
                state.original_screenshot.clone()
            } else {
                state.final_image.clone()
            };

            if let Some(pixbuf) = pixbuf_opt {
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

                // Update editor display transform
                state
                    .editor
                    .update_display_transform(scale, offset_x, offset_y);

                cr.save().expect("Failed to save cairo context");
                cr.translate(offset_x, offset_y);
                cr.scale(scale, scale);
                cr.set_source_pixbuf(&pixbuf, 0.0, 0.0);
                cr.paint().expect("Failed to paint pixbuf");
                cr.restore().expect("Failed to restore cairo context");

                // Draw Selection Overlay only if in capture selection mode
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

                // Draw crop overlay when in crop mode
                if state.is_crop_mode {
                    if let Some((x, y, w, h)) = state.editor.tool_state.get_drag_rect() {
                        // Dimming outside crop area
                        let (dx, dy) = state.editor.image_to_display_coords(x, y);
                        let dw = w * scale;
                        let dh = h * scale;

                        cr.set_source_rgba(0.0, 0.0, 0.0, 0.5);
                        // Top
                        cr.rectangle(0.0, 0.0, da_width, dy);
                        // Bottom
                        cr.rectangle(0.0, dy + dh, da_width, da_height - (dy + dh));
                        // Left
                        cr.rectangle(0.0, dy, dx, dh);
                        // Right
                        cr.rectangle(dx + dw, dy, da_width - (dx + dw), dh);
                        let _ = cr.fill();

                        // Border
                        cr.set_source_rgb(1.0, 1.0, 1.0);
                        cr.set_line_width(2.0);
                        cr.rectangle(dx, dy, dw, dh);
                        let _ = cr.stroke();
                    }
                }

                // Draw annotations (only when not in capture selection mode)
                if !state.is_active {
                    state.editor.draw_annotations(cr);
                }

                // Draw pending text cursor
                if let Some(ref pending) = state.editor.pending_text {
                    let (dx, dy) = state.editor.image_to_display_coords(pending.x, pending.y);
                    cr.set_source_rgba(1.0, 1.0, 1.0, 0.8);
                    cr.set_line_width(2.0);
                    cr.move_to(dx, dy - 20.0);
                    cr.line_to(dx, dy + 5.0);
                    let _ = cr.stroke();
                }
            }
        }
    });

    // Color chooser dialog button
    let color_button = gtk::ColorDialogButton::builder()
        .dialog(&gtk::ColorDialog::new())
        .rgba(&gtk::gdk::RGBA::new(1.0, 0.0, 0.0, 1.0))
        .tooltip_text("Select Color")
        .build();

    // Color picker circle
    let color_picker_circle = gtk::DrawingArea::builder()
        .width_request(20)
        .height_request(20)
        .build();

    color_picker_circle.set_draw_func({
        let state = state.clone();
        move |_, cr, width, height| {
            let state = state.borrow();
            let color = state.editor.current_color();

            // Draw circle
            cr.arc(
                width as f64 / 2.0,
                height as f64 / 2.0,
                (width as f64 / 2.0) - 2.0,
                0.0,
                2.0 * std::f64::consts::PI,
            );
            cr.set_source_rgba(
                color.red() as f64,
                color.green() as f64,
                color.blue() as f64,
                color.alpha() as f64,
            );
            let _ = cr.fill_preserve();
            cr.set_source_rgb(0.0, 0.0, 0.0);
            cr.set_line_width(1.0);
            let _ = cr.stroke();
        }
    });

    color_button.connect_rgba_notify({
        let state = state.clone();
        let color_picker_circle = color_picker_circle.clone();
        move |btn| {
            let color = btn.rgba();
            state.borrow_mut().editor.set_color(color);
            color_picker_circle.queue_draw();
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

    // Tool buttons
    let tool_pointer_btn = gtk::ToggleButton::builder()
        .icon_name("input-mouse-symbolic")
        .tooltip_text("Pointer")
        .active(true)
        .build();
    tool_pointer_btn.add_css_class("flat");

    let tool_pencil_btn = gtk::ToggleButton::builder()
        .icon_name("document-edit-symbolic")
        .tooltip_text("Free Draw")
        .group(&tool_pointer_btn)
        .build();
    tool_pencil_btn.add_css_class("flat");

    let tool_rectangle_btn = gtk::ToggleButton::builder()
        .icon_name("media-playback-stop-symbolic")
        .tooltip_text("Rectangle")
        .group(&tool_pointer_btn)
        .build();
    tool_rectangle_btn.add_css_class("flat");

    let tool_crop_btn = gtk::ToggleButton::builder()
        .icon_name("crop-symbolic")
        .tooltip_text("Crop")
        .group(&tool_pointer_btn)
        .build();
    tool_crop_btn.add_css_class("flat");

    let tool_text_btn = gtk::ToggleButton::builder()
        .icon_name("insert-text-symbolic")
        .tooltip_text("Add Text")
        .group(&tool_pointer_btn)
        .build();
    tool_text_btn.add_css_class("flat");

    let tool_buttons_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .homogeneous(true)
        .build();
    tool_buttons_box.add_css_class("tool-buttons");

    tool_buttons_box.append(&tool_pointer_btn);
    tool_buttons_box.append(&tool_pencil_btn);
    tool_buttons_box.append(&tool_rectangle_btn);
    tool_buttons_box.append(&tool_crop_btn);
    tool_buttons_box.append(&tool_text_btn);
    tool_buttons_box.append(&color_button);

    tools_box.append(&tool_buttons_box);

    // Separator
    let separator = gtk::Separator::builder()
        .orientation(Orientation::Vertical)
        .margin_start(6)
        .margin_end(6)
        .build();

    separator.add_css_class("spacer");

    // Separator
    let separator2 = gtk::Separator::builder()
        .orientation(Orientation::Vertical)
        .margin_start(6)
        .margin_end(6)
        .build();

    separator2.add_css_class("spacer");

    // Undo button
    let undo_btn = gtk::Button::builder()
        .icon_name("edit-undo-symbolic")
        .tooltip_text("Undo")
        .build();
    undo_btn.add_css_class("flat");
    tools_box.append(&undo_btn);

    // Copy to clipboard button
    let copy_btn = gtk::Button::builder()
        .icon_name("edit-copy-symbolic")
        .tooltip_text("Copy to Clipboard")
        .build();
    copy_btn.add_css_class("flat");
    tools_box.append(&copy_btn);

    // Save button
    let save_btn = gtk::Button::builder()
        .icon_name("document-save-symbolic")
        .tooltip_text("Save")
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
        .tooltip_text("Confirm")
        .build();
    confirm_btn.add_css_class("suggested-action");

    crop_tools_box.append(&cancel_btn);
    crop_tools_box.append(&confirm_btn);

    // --- Text Input Popover ---
    let text_entry = gtk::Entry::builder()
        .placeholder_text("Enter text...")
        .width_chars(20)
        .build();

    let text_confirm_btn = gtk::Button::builder()
        .icon_name("object-select-symbolic")
        .tooltip_text("Add Text")
        .build();
    text_confirm_btn.add_css_class("suggested-action");

    let text_cancel_btn = gtk::Button::builder()
        .icon_name("process-stop-symbolic")
        .tooltip_text("Cancel")
        .build();

    let text_input_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();
    text_input_box.append(&text_entry);
    text_input_box.append(&text_confirm_btn);
    text_input_box.append(&text_cancel_btn);

    let text_popover = gtk::Popover::builder()
        .child(&text_input_box)
        .autohide(false)
        .build();
    text_popover.set_parent(&drawing_area);

    // --- Picked Color Display ---
    let picked_color_label = gtk::Label::builder()
        .label("")
        .halign(Align::Center)
        .valign(Align::Start)
        .margin_top(12)
        .visible(false)
        .build();
    picked_color_label.add_css_class("osd");

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
    overlay.add_overlay(&picked_color_label);

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

    // --- Tool Button Connections ---

    tool_pointer_btn.connect_toggled({
        let state = state.clone();
        move |btn| {
            if btn.is_active() {
                let mut s = state.borrow_mut();
                s.editor.set_tool(EditorTool::Pointer);
                s.is_crop_mode = false;
            }
        }
    });

    tool_pencil_btn.connect_toggled({
        let state = state.clone();
        move |btn| {
            if btn.is_active() {
                let mut s = state.borrow_mut();
                s.editor.set_tool(EditorTool::Pencil);
                s.is_crop_mode = false;
            }
        }
    });

    tool_rectangle_btn.connect_toggled({
        let state = state.clone();
        move |btn| {
            if btn.is_active() {
                let mut s = state.borrow_mut();
                s.editor.set_tool(EditorTool::Rectangle);
                s.is_crop_mode = false;
            }
        }
    });

    tool_crop_btn.connect_toggled({
        let state = state.clone();
        let tools_box = tools_box.clone();
        let crop_tools_box = crop_tools_box.clone();
        move |btn| {
            if btn.is_active() {
                let mut s = state.borrow_mut();
                s.editor.set_tool(EditorTool::Crop);
                s.is_crop_mode = true;
                drop(s);
                tools_box.set_visible(false);
                crop_tools_box.set_visible(true);
            }
        }
    });

    tool_text_btn.connect_toggled({
        let state = state.clone();
        move |btn| {
            if btn.is_active() {
                let mut s = state.borrow_mut();
                s.editor.set_tool(EditorTool::Text);
                s.is_crop_mode = false;
            }
        }
    });

    // --- Undo Button ---
    undo_btn.connect_clicked({
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        move |_| {
            let mut s = state.borrow_mut();
            s.editor.undo();
            drop(s);
            drawing_area.queue_draw();
        }
    });

    // --- Copy to Clipboard Button ---
    copy_btn.connect_clicked({
        let state = state.clone();
        let window = window.clone();
        move |_| {
            let s = state.borrow();
            if let Some(ref pixbuf) = s.final_image {
                let clipboard_manager = ClipboardManager::from_widget(&window);
                if clipboard_manager.copy_image(pixbuf).is_ok() {
                    println!("Image copied to clipboard");
                }
            }
        }
    });

    // --- Drag Controller for Drawing ---
    let drag = GestureDrag::new();

    drag.connect_drag_begin({
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        move |_, x, y| {
            let mut s = state.borrow_mut();

            // Handle capture selection mode
            if s.is_active && s.mode == CaptureMode::Selection {
                s.selection = Some(Selection {
                    start_x: x,
                    start_y: y,
                    end_x: x,
                    end_y: y,
                });
                drop(s);
                drawing_area.queue_draw();
                return;
            }

            // Handle editor tools
            if s.final_image.is_some() {
                let tool = s.editor.current_tool();
                match tool {
                    EditorTool::Pencil => {
                        let (img_x, img_y) = s.editor.display_to_image_coords(x, y);
                        s.editor.tool_state.start_drag(img_x, img_y);
                        let mut free_draw = FreeDrawAnnotation::new(
                            s.editor.tool_state.color,
                            s.editor.tool_state.line_width,
                        );
                        free_draw.add_point(img_x, img_y);
                        s.editor
                            .annotations
                            .set_current(Some(Annotation::FreeDraw(free_draw)));
                    }
                    EditorTool::Rectangle => {
                        let (img_x, img_y) = s.editor.display_to_image_coords(x, y);
                        s.editor.tool_state.start_drag(img_x, img_y);
                    }
                    EditorTool::Crop => {
                        let (img_x, img_y) = s.editor.display_to_image_coords(x, y);
                        s.editor.tool_state.start_drag(img_x, img_y);
                    }
                    _ => {}
                }
                drop(s);
                drawing_area.queue_draw();
            }
        }
    });

    drag.connect_drag_update({
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        move |gesture, offset_x, offset_y| {
            let mut s = state.borrow_mut();

            // Handle capture selection mode
            if s.is_active && s.mode == CaptureMode::Selection {
                if let Some(sel) = &mut s.selection {
                    sel.end_x = sel.start_x + offset_x;
                    sel.end_y = sel.start_y + offset_y;
                    drop(s);
                    drawing_area.queue_draw();
                }
                return;
            }

            // Handle editor tools
            if s.final_image.is_some() {
                let tool = s.editor.current_tool();

                if let Some((start_x, start_y)) = gesture.start_point() {
                    let current_x = start_x + offset_x;
                    let current_y = start_y + offset_y;
                    let (img_x, img_y) = s.editor.display_to_image_coords(current_x, current_y);

                    match tool {
                        EditorTool::Pencil => {
                            s.editor.tool_state.update_drag(img_x, img_y);
                            if let Some(Annotation::FreeDraw(ref draw)) =
                                s.editor.annotations.current().cloned()
                            {
                                let mut draw = draw.clone();
                                draw.add_point(img_x, img_y);
                                s.editor
                                    .annotations
                                    .set_current(Some(Annotation::FreeDraw(draw)));
                            }
                        }
                        EditorTool::Rectangle => {
                            s.editor.tool_state.update_drag(img_x, img_y);
                            if let (Some((start_x, start_y)), Some((end_x, end_y))) = (
                                s.editor.tool_state.drag_start,
                                s.editor.tool_state.drag_current,
                            ) {
                                let rect = RectangleAnnotation::from_corners(
                                    start_x,
                                    start_y,
                                    end_x,
                                    end_y,
                                    s.editor.tool_state.color,
                                    s.editor.tool_state.line_width,
                                );
                                s.editor
                                    .annotations
                                    .set_current(Some(Annotation::Rectangle(rect)));
                            }
                        }
                        EditorTool::Crop => {
                            s.editor.tool_state.update_drag(img_x, img_y);
                        }
                        _ => {}
                    }
                }
                drop(s);
                drawing_area.queue_draw();
            }
        }
    });

    drag.connect_drag_end({
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        move |gesture, offset_x, offset_y| {
            let mut s = state.borrow_mut();

            // Handle capture selection mode
            if s.is_active && s.mode == CaptureMode::Selection {
                if let Some(sel) = &mut s.selection {
                    sel.end_x = sel.start_x + offset_x;
                    sel.end_y = sel.start_y + offset_y;
                    drop(s);
                    drawing_area.queue_draw();
                }
                return;
            }

            // Handle editor tools
            if s.final_image.is_some() {
                let tool = s.editor.current_tool();

                if let Some((start_x, start_y)) = gesture.start_point() {
                    let current_x = start_x + offset_x;
                    let current_y = start_y + offset_y;
                    let (img_x, img_y) = s.editor.display_to_image_coords(current_x, current_y);

                    match tool {
                        EditorTool::Pencil => {
                            s.editor.tool_state.update_drag(img_x, img_y);
                            s.editor.annotations.commit_current();
                            s.editor.tool_state.end_drag();
                        }
                        EditorTool::Rectangle => {
                            s.editor.tool_state.update_drag(img_x, img_y);
                            s.editor.annotations.commit_current();
                            s.editor.tool_state.end_drag();
                        }
                        EditorTool::Crop => {
                            // Keep the drag state for crop confirmation
                            s.editor.tool_state.update_drag(img_x, img_y);
                        }
                        _ => {}
                    }
                }
                drop(s);
                drawing_area.queue_draw();
            }
        }
    });

    drawing_area.add_controller(drag);

    // --- Click Controller for Color Picker and Text ---
    let click = GestureClick::new();
    click.set_button(1); // Left click

    click.connect_released({
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        let picked_color_label = picked_color_label.clone();
        let color_button = color_button.clone();
        let text_popover = text_popover.clone();
        let text_entry = text_entry.clone();
        move |_, _, x, y| {
            let mut s = state.borrow_mut();

            if s.final_image.is_none() || s.is_active {
                return;
            }

            let tool = s.editor.current_tool();

            match tool {
                EditorTool::ColorPicker => {
                    if let Some(ref pixbuf) = s.final_image.clone() {
                        let (img_x, img_y) = s.editor.display_to_image_coords(x, y);
                        if let Ok(picked) =
                            pick_color_from_pixbuf(pixbuf, img_x as i32, img_y as i32)
                        {
                            let color = picked.color;
                            let hex = picked.to_hex();
                            s.editor.set_color(color);
                            s.editor.color_picker.set_picked_color(picked);
                            drop(s);

                            // Update UI
                            picked_color_label.set_text(&format!("Color: {}", hex));
                            color_button.set_rgba(&color);
                            color_picker_circle.queue_draw();
                            drawing_area.queue_draw();
                        }
                    }
                }
                EditorTool::Text => {
                    let (img_x, img_y) = s.editor.display_to_image_coords(x, y);
                    s.editor.pending_text = Some(editor::PendingText {
                        x: img_x,
                        y: img_y,
                        text: String::new(),
                    });
                    drop(s);

                    // Show text input popover
                    text_entry.set_text("");
                    let rect = gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1);
                    text_popover.set_pointing_to(Some(&rect));
                    text_popover.popup();
                    text_entry.grab_focus();

                    drawing_area.queue_draw();
                }
                _ => {}
            }
        }
    });

    drawing_area.add_controller(click);

    // --- Text Input Handling ---
    text_confirm_btn.connect_clicked({
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        let text_popover = text_popover.clone();
        let text_entry = text_entry.clone();
        move |_| {
            let text = text_entry.text().to_string();
            let mut s = state.borrow_mut();
            s.editor.commit_text(text);
            drop(s);
            text_popover.popdown();
            drawing_area.queue_draw();
        }
    });

    text_cancel_btn.connect_clicked({
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        let text_popover = text_popover.clone();
        move |_| {
            let mut s = state.borrow_mut();
            s.editor.cancel_text();
            drop(s);
            text_popover.popdown();
            drawing_area.queue_draw();
        }
    });

    text_entry.connect_activate({
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        let text_popover = text_popover.clone();
        let text_entry = text_entry.clone();
        move |_| {
            let text = text_entry.text().to_string();
            let mut s = state.borrow_mut();
            s.editor.commit_text(text);
            drop(s);
            text_popover.popdown();
            drawing_area.queue_draw();
        }
    });

    // --- Crop Confirm/Cancel ---
    confirm_btn.connect_clicked({
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        let window = window.clone();
        let header_bar = header_bar.clone();
        let tools_box = tools_box.clone();
        let crop_tools_box = crop_tools_box.clone();
        let tool_pointer_btn = tool_pointer_btn.clone();

        move |_| {
            let mut s = state.borrow_mut();

            // Check if we're in capture selection mode
            if s.is_active && s.mode == CaptureMode::Selection {
                if let Some(sel) = s.selection {
                    let rect = sel.rectangle();

                    // Only crop if area is significant
                    if rect.width() > 10 && rect.height() > 10 {
                        if let Some(orig) = &s.original_screenshot {
                            // Assuming scale 1.0 for fullscreen capture
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

                // Exit capture selection mode
                s.is_active = false;
                s.selection = None;
                s.editor.reset();

                // Restore UI
                window.unfullscreen();
                header_bar.set_visible(true);
                tools_box.set_visible(true);
                crop_tools_box.set_visible(false);

                drop(s);
                drawing_area.queue_draw();
                return;
            }

            // Handle editor crop mode
            if s.is_crop_mode {
                if let Some((x, y, w, h)) = s.editor.tool_state.get_drag_rect() {
                    if w > 10.0 && h > 10.0 {
                        if let Some(ref pixbuf) = s.final_image.clone() {
                            let crop_x = (x as i32).max(0);
                            let crop_y = (y as i32).max(0);
                            let crop_w = (w as i32).min(pixbuf.width() - crop_x);
                            let crop_h = (h as i32).min(pixbuf.height() - crop_y);

                            if crop_w > 0 && crop_h > 0 {
                                let cropped = pixbuf.new_subpixbuf(crop_x, crop_y, crop_w, crop_h);
                                s.final_image = Some(cropped);
                                s.editor.clear_annotations(); // Clear annotations after crop
                            }
                        }
                    }
                }

                s.is_crop_mode = false;
                s.editor.tool_state.reset_drag();
                s.editor.set_tool(EditorTool::Pointer);
                drop(s);

                tool_pointer_btn.set_active(true);
                tools_box.set_visible(true);
                crop_tools_box.set_visible(false);
                drawing_area.queue_draw();
            }
        }
    });

    cancel_btn.connect_clicked({
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        let window = window.clone();
        let header_bar = header_bar.clone();
        let tools_box = tools_box.clone();
        let crop_tools_box = crop_tools_box.clone();
        let placeholder_icon = placeholder_icon.clone();
        let tool_pointer_btn = tool_pointer_btn.clone();

        move |_| {
            let mut s = state.borrow_mut();

            // Check if we're in capture selection mode
            if s.is_active {
                s.is_active = false;
                s.selection = None;

                window.unfullscreen();
                header_bar.set_visible(true);
                tools_box.set_visible(true);
                crop_tools_box.set_visible(false);

                if s.final_image.is_none() {
                    placeholder_icon.set_visible(true);
                }

                drop(s);
                drawing_area.queue_draw();
                return;
            }

            // Handle editor crop mode cancel
            if s.is_crop_mode {
                s.is_crop_mode = false;
                s.editor.tool_state.reset_drag();
                s.editor.set_tool(EditorTool::Pointer);
                drop(s);

                tool_pointer_btn.set_active(true);
                tools_box.set_visible(true);
                crop_tools_box.set_visible(false);
                drawing_area.queue_draw();
            }
        }
    });

    // --- Take Screenshot Action ---
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
                            s.editor.reset();

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
                        s.editor.reset();

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
