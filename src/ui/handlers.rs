use gtk::glib;
use gtk4 as gtk;
use libadwaita as adw;
use log::{debug, error, info};

use gtk::{GestureClick, GestureDrag};
use gtk4::prelude::*;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::app::{AppState, CaptureMode};
use crate::capture::capture_primary_monitor;
use crate::editor::{
    pick_color_from_pixbuf, Annotation, ClipboardManager, EditorTool, FreeDrawAnnotation,
    RectangleAnnotation,
};
use crate::ui::dialogs::{show_window_selector, TextPopoverComponents};
use crate::ui::drawing::DrawingComponents;
use crate::ui::header::HeaderComponents;
use crate::ui::toolbar::{CropToolbarComponents, ToolbarComponents};

pub struct UiComponents {
    pub window: adw::ApplicationWindow,
    pub header: HeaderComponents,
    pub toolbar: ToolbarComponents,
    pub crop_toolbar: CropToolbarComponents,
    pub drawing: DrawingComponents,
    pub text_popover: TextPopoverComponents,
}

pub fn connect_undo_handler(state: &Rc<RefCell<AppState>>, components: &UiComponents) {
    components.toolbar.undo_btn.connect_clicked({
        let state = state.clone();
        let drawing_area = components.drawing.drawing_area.clone();
        move |_| {
            debug!("Undo requested");
            let mut s = state.borrow_mut();
            s.editor.undo();
            drop(s);
            drawing_area.queue_draw();
        }
    });
}

pub fn connect_copy_handler(state: &Rc<RefCell<AppState>>, components: &UiComponents) {
    components.toolbar.copy_btn.connect_clicked({
        let state = state.clone();
        let window = components.window.clone();
        move |_| {
            let s = state.borrow();
            if let Some(ref pixbuf) = s.final_image {
                let clipboard_manager = ClipboardManager::from_widget(&window);
                if clipboard_manager.copy_image(pixbuf).is_ok() {
                    info!("Image copied to clipboard");
                }
            }
        }
    });
}

pub fn connect_save_handler(state: &Rc<RefCell<AppState>>, components: &UiComponents) {
    components.toolbar.save_btn.connect_clicked({
        let state = state.clone();
        let window = components.window.clone();
        move |_| {
            let state = state.clone();
            let window = window.clone();
            glib::spawn_future_local(async move {
                let dialog = gtk::FileDialog::new();
                match dialog.select_folder_future(Some(&window)).await {
                    Ok(folder) => {
                        if let Some(folder_path) = folder.path() {
                            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH);

                            let value_in_secs_timestamp = match timestamp {
                                Ok(dur) => dur.as_secs(),
                                Err(_) => 0,
                            };

                            let mut path = PathBuf::from(folder_path);
                            path.push(format!("screenshot_{}.png", value_in_secs_timestamp));
                            let s = state.borrow();
                            if let Some(ref pixbuf) = s.final_image {
                                if let Err(e) = pixbuf.savev(path.to_str().unwrap(), "png", &[]) {
                                    error!("Failed to save image: {}", e);
                                } else {
                                    info!("Image saved to {:?}", path);
                                }
                            }
                        }
                    }
                    Err(_) => {}
                }
            });
        }
    });
}

pub fn connect_drag_handlers(state: &Rc<RefCell<AppState>>, components: &UiComponents) {
    debug!("Connecting drag handlers");
    let drag = GestureDrag::new();

    drag.connect_drag_begin({
        let state = state.clone();
        let drawing_area = components.drawing.drawing_area.clone();
        move |_, x, y| {
            handle_drag_begin(&state, &drawing_area, x, y);
        }
    });

    drag.connect_drag_update({
        let state = state.clone();
        let drawing_area = components.drawing.drawing_area.clone();
        move |gesture, offset_x, offset_y| {
            handle_drag_update(&state, &drawing_area, gesture, offset_x, offset_y);
        }
    });

    drag.connect_drag_end({
        let state = state.clone();
        let drawing_area = components.drawing.drawing_area.clone();
        move |gesture, offset_x, offset_y| {
            handle_drag_end(&state, &drawing_area, gesture, offset_x, offset_y);
        }
    });

    components.drawing.drawing_area.add_controller(drag);
}

fn handle_drag_begin(
    state: &Rc<RefCell<AppState>>,
    drawing_area: &gtk::DrawingArea,
    x: f64,
    y: f64,
) {
    let mut s = state.borrow_mut();

    if s.is_active && s.mode == CaptureMode::Selection {
        s.start_selection(x, y);
        drop(s);
        drawing_area.queue_draw();
        return;
    }

    if s.final_image.is_some() {
        let tool = s.editor.current_tool();
        match tool {
            EditorTool::Pointer | EditorTool::Text => {
                s.editor.pointer_drag_start(x, y);
            }
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

fn handle_drag_update(
    state: &Rc<RefCell<AppState>>,
    drawing_area: &gtk::DrawingArea,
    gesture: &GestureDrag,
    offset_x: f64,
    offset_y: f64,
) {
    let mut s = state.borrow_mut();

    if s.is_active && s.mode == CaptureMode::Selection {
        if let Some(ref sel) = s.selection {
            let end_x = sel.start_x + offset_x;
            let end_y = sel.start_y + offset_y;
            s.update_selection(end_x, end_y);
            drop(s);
            drawing_area.queue_draw();
        }
        return;
    }

    if s.final_image.is_some() {
        let tool = s.editor.current_tool();

        if let Some((start_x, start_y)) = gesture.start_point() {
            let current_x = start_x + offset_x;
            let current_y = start_y + offset_y;
            let (img_x, img_y) = s.editor.display_to_image_coords(current_x, current_y);

            match tool {
                EditorTool::Pointer | EditorTool::Text => {
                    s.editor.pointer_drag_update(current_x, current_y);
                }
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

fn handle_drag_end(
    state: &Rc<RefCell<AppState>>,
    drawing_area: &gtk::DrawingArea,
    gesture: &GestureDrag,
    offset_x: f64,
    offset_y: f64,
) {
    let mut s = state.borrow_mut();

    if s.is_active && s.mode == CaptureMode::Selection {
        if let Some(ref sel) = s.selection {
            let end_x = sel.start_x + offset_x;
            let end_y = sel.start_y + offset_y;
            s.update_selection(end_x, end_y);
            drop(s);
            drawing_area.queue_draw();
        }
        return;
    }

    if s.final_image.is_some() {
        let tool = s.editor.current_tool();

        if let Some((start_x, start_y)) = gesture.start_point() {
            let current_x = start_x + offset_x;
            let current_y = start_y + offset_y;
            let (img_x, img_y) = s.editor.display_to_image_coords(current_x, current_y);

            match tool {
                EditorTool::Pointer | EditorTool::Text => {
                    s.editor.pointer_drag_end();
                }
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
                    s.editor.tool_state.update_drag(img_x, img_y);
                }
                _ => {}
            }
        }
        drop(s);
        drawing_area.queue_draw();
    }
}

pub fn connect_click_handlers(state: &Rc<RefCell<AppState>>, components: &UiComponents) {
    let click = GestureClick::new();
    click.set_button(1);

    click.connect_released({
        let state = state.clone();
        let drawing_area = components.drawing.drawing_area.clone();
        let picked_color_label = components.drawing.picked_color_label.clone();
        let color_button = components.toolbar.color_button.clone();
        let color_picker_circle = components.toolbar.color_picker_circle.clone();
        let text_popover = components.text_popover.text_popover.clone();
        let text_entry = components.text_popover.text_entry.clone();
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
                            pick_color_from_pixbuf(&pixbuf, img_x as i32, img_y as i32)
                        {
                            let color = picked.color;
                            let hex = picked.to_hex();
                            s.editor.set_color(color);
                            s.editor.color_picker.set_picked_color(picked);
                            drop(s);

                            picked_color_label.set_text(&format!("Color: {}", hex));
                            color_button.set_rgba(&color);
                            color_picker_circle.queue_draw();
                            drawing_area.queue_draw();
                        }
                    }
                }
                EditorTool::Text => {
                    if s.editor.last_drag_moved {
                        return;
                    }

                    let (img_x, img_y) = s.editor.display_to_image_coords(x, y);

                    // If we clicked on an existing annotation, just select it and don't open popover
                    if let Some(index) = s.editor.annotations.hit_test(img_x, img_y) {
                        s.editor.annotations.set_selected(Some(index));
                        drop(s);
                        drawing_area.queue_draw();
                        return;
                    }

                    s.editor.pending_text = Some(crate::editor::PendingText { x: img_x, y: img_y });
                    drop(s);

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

    components.drawing.drawing_area.add_controller(click);
}

pub fn connect_crop_handlers(state: &Rc<RefCell<AppState>>, components: &UiComponents) {
    components.crop_toolbar.confirm_btn.connect_clicked({
        let state = state.clone();
        let drawing_area = components.drawing.drawing_area.clone();
        let window = components.window.clone();
        let header_bar = components.header.header_bar.clone();
        let tools_box = components.toolbar.tools_box.clone();
        let crop_tools_box = components.crop_toolbar.crop_tools_box.clone();
        let tool_pointer_btn = components.toolbar.tool_pointer_btn.clone();

        move |_| {
            let mut s = state.borrow_mut();

            if s.is_active && s.mode == CaptureMode::Selection {
                s.apply_selection_crop();
                s.exit_capture_mode();

                window.unfullscreen();
                header_bar.set_visible(true);
                tools_box.set_visible(true);
                crop_tools_box.set_visible(false);

                drop(s);
                drawing_area.queue_draw();
                return;
            }

            if s.is_crop_mode {
                s.apply_editor_crop();
                s.exit_crop_mode();
                s.editor.set_tool(EditorTool::Pointer);
                drop(s);

                tool_pointer_btn.set_active(true);
                tools_box.set_visible(true);
                crop_tools_box.set_visible(false);
                drawing_area.queue_draw();
            }
        }
    });

    components.crop_toolbar.cancel_btn.connect_clicked({
        let state = state.clone();
        let drawing_area = components.drawing.drawing_area.clone();
        let window = components.window.clone();
        let header_bar = components.header.header_bar.clone();
        let tools_box = components.toolbar.tools_box.clone();
        let crop_tools_box = components.crop_toolbar.crop_tools_box.clone();
        let placeholder_icon = components.drawing.placeholder_icon.clone();
        let tool_pointer_btn = components.toolbar.tool_pointer_btn.clone();

        move |_| {
            let mut s = state.borrow_mut();

            if s.is_active {
                s.is_active = false;
                s.selection = None;

                window.unfullscreen();
                header_bar.set_visible(true);
                tools_box.set_visible(s.final_image.is_some());
                crop_tools_box.set_visible(false);

                if s.final_image.is_none() {
                    placeholder_icon.set_visible(true);
                }

                drop(s);
                drawing_area.queue_draw();
                return;
            }

            if s.is_crop_mode {
                s.exit_crop_mode();
                s.editor.set_tool(EditorTool::Pointer);
                drop(s);

                tool_pointer_btn.set_active(true);
                tools_box.set_visible(true);
                crop_tools_box.set_visible(false);
                drawing_area.queue_draw();
            }
        }
    });
}

pub fn connect_screenshot_handler(state: &Rc<RefCell<AppState>>, components: &UiComponents) {
    components.header.take_screenshot_btn.connect_clicked({
        let state = state.clone();
        let drawing_area = components.drawing.drawing_area.clone();
        let placeholder_icon = components.drawing.placeholder_icon.clone();
        let window = components.window.clone();
        let header_bar = components.header.header_bar.clone();
        let tools_box = components.toolbar.tools_box.clone();
        let crop_tools_box = components.crop_toolbar.crop_tools_box.clone();

        move |_| {
            tools_box.set_visible(false);
            let mode = state.borrow().mode;

            if mode == CaptureMode::Window {
                show_window_selector(
                    &state,
                    &window,
                    &drawing_area,
                    &placeholder_icon,
                    &tools_box,
                );
            } else {
                capture_screen_or_selection(
                    &state,
                    &window,
                    &header_bar,
                    &tools_box,
                    &crop_tools_box,
                    &drawing_area,
                    &placeholder_icon,
                    mode,
                );
            }
        }
    });
}

pub fn capture_screen_or_selection(
    state: &Rc<RefCell<AppState>>,
    window: &adw::ApplicationWindow,
    header_bar: &adw::HeaderBar,
    tools_box: &gtk::Box,
    crop_tools_box: &gtk::Box,
    drawing_area: &gtk::DrawingArea,
    placeholder_icon: &gtk::Image,
    mode: CaptureMode,
) {
    window.set_visible(false);

    let context = gtk::glib::MainContext::default();
    while context.pending() {
        context.iteration(false);
    }
    std::thread::sleep(Duration::from_millis(200));

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
                tools_box.set_visible(true);
            } else {
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
            error!("Failed to capture screen: {}", e);
            window.set_visible(true);
        }
    }
}

pub fn connect_all_handlers(state: &Rc<RefCell<AppState>>, components: &UiComponents) {
    debug!("Initializing UI handlers");
    connect_undo_handler(state, components);
    connect_copy_handler(state, components);
    connect_save_handler(state, components);
    connect_drag_handlers(state, components);
    connect_click_handlers(state, components);
    connect_crop_handlers(state, components);
    connect_screenshot_handler(state, components);
}
