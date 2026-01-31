use gtk::glib;
use gtk4 as gtk;
use libadwaita as adw;
use log::{debug, error, info};

use gtk::gio;
use gtk::prelude::*;
use gtk::{EventControllerKey, GestureClick, GestureDrag};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::app::config::Action;
use crate::app::{AppState, CaptureMode};
use crate::capture::capture_primary_monitor;
use crate::editor::{
    pick_color_from_pixbuf, Annotation, ClipboardManager, EditorTool, FreeDrawAnnotation,
    RectangleAnnotation,
};
use crate::ui::dialogs::{show_about_dialog, show_window_selector, TextPopoverComponents};
use crate::ui::drawing::DrawingComponents;
use crate::ui::header::HeaderComponents;
use crate::ui::shortcuts;
use crate::ui::toolbar::{CropToolbarComponents, SelectionToolbarComponents, ToolbarComponents};

#[derive(Clone)]
pub struct UiComponents {
    pub window: adw::ApplicationWindow,
    pub header: HeaderComponents,
    pub toolbar: ToolbarComponents,
    pub crop_toolbar: CropToolbarComponents,
    pub selection_toolbar: SelectionToolbarComponents,
    pub drawing: DrawingComponents,
    pub text_popover: TextPopoverComponents,
}

// Helper functions for actions
fn perform_copy(state: &Rc<RefCell<AppState>>, window: &impl IsA<gtk::Widget>) {
    let s = state.borrow();
    if let Some(ref pixbuf) = s.final_image {
        let clipboard_manager = ClipboardManager::from_widget(window);
        if clipboard_manager.copy_image(pixbuf).is_ok() {
            info!("Image copied to clipboard");
        }
    }
}

fn perform_undo(state: &Rc<RefCell<AppState>>, drawing_area: &gtk::DrawingArea) {
    let mut s = state.borrow_mut();
    if s.editor.undo() {
        drop(s);
        drawing_area.queue_draw();
    }
}

fn perform_save(state: Rc<RefCell<AppState>>, window: impl IsA<gtk::Window> + Clone + 'static) {
    glib::spawn_future_local(async move {
        let dialog = gtk::FileDialog::new();
        if let Ok(folder) = dialog.select_folder_future(Some(&window)).await {
            if let Some(folder_path) = folder.path() {
                let timestamp = SystemTime::now().duration_since(UNIX_EPOCH);
                let value_in_secs_timestamp = match timestamp {
                    Ok(dur) => dur.as_secs(),
                    Err(_) => 0,
                };

                let mut path = folder_path;
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
    });
}

pub fn connect_undo_handler(state: &Rc<RefCell<AppState>>, components: &UiComponents) {
    components.toolbar.undo_btn.connect_clicked({
        let state = state.clone();
        let drawing_area = components.drawing.drawing_area.clone();
        move |_| {
            perform_undo(&state, &drawing_area);
        }
    });
}

pub fn connect_copy_handler(state: &Rc<RefCell<AppState>>, components: &UiComponents) {
    components.toolbar.copy_btn.connect_clicked({
        let state = state.clone();
        let window = components.window.clone();
        move |_| {
            perform_copy(&state, &window);
        }
    });
}

pub fn connect_save_handler(state: &Rc<RefCell<AppState>>, components: &UiComponents) {
    components.toolbar.save_btn.connect_clicked({
        let state = state.clone();
        let window = components.window.clone();
        move |_| {
            perform_save(state.clone(), window.clone());
        }
    });
}

pub fn connect_drag_handlers(state: &Rc<RefCell<AppState>>, components: &UiComponents) {
    let drag = GestureDrag::new();
    drag.set_button(1); // Left mouse button

    drag.connect_drag_begin({
        let state = state.clone();
        let drawing_area = components.drawing.drawing_area.clone();
        move |gesture, x, y| {
            handle_drag_begin(&state, gesture, x, y);
            drawing_area.queue_draw();
        }
    });

    drag.connect_drag_update({
        let state = state.clone();
        let drawing_area = components.drawing.drawing_area.clone();
        move |gesture, x, y| {
            handle_drag_update(&state, gesture, x, y);
            drawing_area.queue_draw();
        }
    });

    drag.connect_drag_end({
        let state = state.clone();
        let drawing_area = components.drawing.drawing_area.clone();
        move |gesture, x, y| {
            handle_drag_end(&state, gesture, x, y);
            drawing_area.queue_draw();
        }
    });

    components.drawing.drawing_area.add_controller(drag);
}

fn handle_drag_begin(
    state: &Rc<RefCell<AppState>>,
    _gesture: &GestureDrag,
    start_x: f64,
    start_y: f64,
) {
    let mut s = state.borrow_mut();
    if s.is_active && s.mode == CaptureMode::Selection {
        s.start_selection(start_x, start_y);
    } else if s.final_image.is_some() {
        if s.editor.pointer_drag_start(start_x, start_y) {
            return;
        }

        let (img_x, img_y) = s.editor.display_to_image_coords(start_x, start_y);

        match s.editor.current_tool() {
            EditorTool::Pencil => {
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
                s.editor.tool_state.start_drag(img_x, img_y);
            }
            EditorTool::Crop => {
                // For crop, reset any existing selection when starting a new one
                s.editor.tool_state.reset_drag();
                s.editor.tool_state.start_drag(img_x, img_y);
            }
            _ => {}
        }
    }
}

fn handle_drag_update(
    state: &Rc<RefCell<AppState>>,
    gesture: &GestureDrag,
    offset_x: f64,
    offset_y: f64,
) {
    let mut s = state.borrow_mut();

    let (start_x, start_y) = match gesture.start_point() {
        Some(p) => p,
        None => return,
    };
    let current_x = start_x + offset_x;
    let current_y = start_y + offset_y;

    if s.is_active && s.mode == CaptureMode::Selection {
        s.update_selection(current_x, current_y);
    } else if s.final_image.is_some() {
        let (img_x, img_y) = s.editor.display_to_image_coords(current_x, current_y);

        if s.editor.tool_state.is_dragging_annotation {
            s.editor.pointer_drag_update(current_x, current_y);
        } else if s.editor.tool_state.is_drawing {
            s.editor.tool_state.update_drag(img_x, img_y);

            if s.editor.current_tool() == EditorTool::Pencil {
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
        }
    }
}

fn handle_drag_end(
    state: &Rc<RefCell<AppState>>,
    gesture: &GestureDrag,
    offset_x: f64,
    offset_y: f64,
) {
    let mut s = state.borrow_mut();
    let (start_x, start_y) = match gesture.start_point() {
        Some(p) => p,
        None => return,
    };
    let current_x = start_x + offset_x;
    let current_y = start_y + offset_y;

    if s.is_active && s.mode == CaptureMode::Selection {
        s.update_selection(current_x, current_y);
    } else if s.final_image.is_some() {
        if s.editor.tool_state.is_dragging_annotation {
            s.editor.pointer_drag_end();
        } else if s.editor.tool_state.is_drawing {
            let tool = s.editor.current_tool();

            if tool == EditorTool::Pencil {
                s.editor.tool_state.end_drag();
                s.editor.annotations.commit_current();
            } else if tool == EditorTool::Rectangle {
                let drag_result = s.editor.tool_state.end_drag();
                if let Some((start, end)) = drag_result {
                    let color = s.editor.tool_state.color;
                    let rect = RectangleAnnotation::new(
                        start.0,
                        start.1,
                        (end.0 - start.0).abs(),
                        (end.1 - start.1).abs(),
                        color,
                        3.0,
                    );
                    s.editor.annotations.add(Annotation::Rectangle(rect));
                }
            } else if tool == EditorTool::Crop {
                // For crop, we keep the drag coordinates in ToolState but stop drawing
                s.editor.tool_state.is_drawing = false;
            }
        }
    }
}

pub fn connect_click_handlers(state: &Rc<RefCell<AppState>>, components: &UiComponents) {
    let click = GestureClick::new();
    click.connect_pressed({
        let state = state.clone();
        let drawing_area = components.drawing.drawing_area.clone();
        let text_popover = components.text_popover.text_popover.clone();
        let text_entry = components.text_popover.text_entry.clone();
        move |_gesture, _n_press, x, y| {
            let mut s = state.borrow_mut();
            if s.final_image.is_some() {
                if s.editor.current_tool() == EditorTool::Text {
                    let rect = gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1);
                    text_popover.set_pointing_to(Some(&rect));
                    text_popover.popup();
                    text_entry.set_text("");
                    text_entry.grab_focus();

                    let (img_x, img_y) = s.editor.display_to_image_coords(x, y);
                    s.editor.pending_text = Some(crate::editor::PendingText { x: img_x, y: img_y });
                } else if s.editor.current_tool() == EditorTool::ColorPicker {
                    let (img_x, img_y) = s.editor.display_to_image_coords(x, y);
                    if let Some(ref pixbuf) = s.final_image {
                        if let Ok(picked) =
                            pick_color_from_pixbuf(pixbuf, img_x as i32, img_y as i32)
                        {
                            s.editor.set_color(picked.color);
                        }
                    }
                }
            }
            drawing_area.queue_draw();
        }
    });
    components.drawing.drawing_area.add_controller(click);
}

fn confirm_selection(
    state: &mut AppState,
    window: &adw::ApplicationWindow,
    header_bar: &adw::HeaderBar,
    tools_box: &gtk::Box,
    crop_tools_box: &gtk::Box,
) -> bool {
    if state.apply_selection_crop() {
        state.is_active = false;
        state.selection = None;
        window.unfullscreen();
        header_bar.set_visible(true);
        tools_box.set_visible(true);
        crop_tools_box.set_visible(false);
        return true;
    }
    false
}

pub fn connect_crop_handlers(state: &Rc<RefCell<AppState>>, components: &UiComponents) {
    components.crop_toolbar.confirm_btn.connect_clicked({
        let state = state.clone();
        let drawing_area = components.drawing.drawing_area.clone();
        let tools_box = components.toolbar.tools_box.clone();
        let crop_tools_box = components.crop_toolbar.crop_tools_box.clone();
        move |_| {
            let mut s = state.borrow_mut();
            if s.apply_editor_crop() {
                s.exit_crop_mode();
                tools_box.set_visible(true);
                crop_tools_box.set_visible(false);
                drawing_area.queue_draw();
            }
        }
    });

    components.crop_toolbar.cancel_btn.connect_clicked({
        let state = state.clone();
        let drawing_area = components.drawing.drawing_area.clone();
        let tools_box = components.toolbar.tools_box.clone();
        let crop_tools_box = components.crop_toolbar.crop_tools_box.clone();
        move |_| {
            let mut s = state.borrow_mut();
            s.exit_crop_mode();
            tools_box.set_visible(true);
            crop_tools_box.set_visible(false);
            drawing_area.queue_draw();
        }
    });
}

pub fn connect_selection_handlers(state: &Rc<RefCell<AppState>>, components: &UiComponents) {
    components.selection_toolbar.confirm_btn.connect_clicked({
        let state = state.clone();
        let window = components.window.clone();
        let header_bar = components.header.header_bar.clone();
        let tools_box = components.toolbar.tools_box.clone();
        let crop_tools_box = components.crop_toolbar.crop_tools_box.clone();
        let selection_tools_box = components.selection_toolbar.selection_tools_box.clone();
        let drawing_area = components.drawing.drawing_area.clone();
        move |_| {
            let mut s = state.borrow_mut();
            if confirm_selection(
                &mut s,
                &window,
                &header_bar,
                &tools_box,
                &crop_tools_box,
            ) {
                selection_tools_box.set_visible(false);
                drop(s);
                drawing_area.queue_draw();
            }
        }
    });

    components.selection_toolbar.cancel_btn.connect_clicked({
        let state = state.clone();
        let window = components.window.clone();
        let header_bar = components.header.header_bar.clone();
        let tools_box = components.toolbar.tools_box.clone();
        let crop_tools_box = components.crop_toolbar.crop_tools_box.clone();
        let selection_tools_box = components.selection_toolbar.selection_tools_box.clone();
        let placeholder_icon = components.drawing.placeholder_icon.clone();
        let drawing_area = components.drawing.drawing_area.clone();
        move |_| {
            let mut s = state.borrow_mut();
            s.exit_capture_mode();
            window.unfullscreen();
            header_bar.set_visible(true);
            tools_box.set_visible(s.final_image.is_some());
            crop_tools_box.set_visible(false);
            selection_tools_box.set_visible(false);
            if s.final_image.is_none() {
                placeholder_icon.set_visible(true);
            }
            drop(s);
            drawing_area.queue_draw();
        }
    });
}

pub fn connect_screenshot_handler(state: &Rc<RefCell<AppState>>, components: &UiComponents) {
    components.header.take_screenshot_btn.connect_clicked({
        let state = state.clone();
        let components = components.clone();
        move |_| {
            let mode = state.borrow().mode;
            capture_screen_or_selection(&state, &components, mode);
        }
    });
}

pub fn connect_keyboard_handlers(state: &Rc<RefCell<AppState>>, components: &UiComponents) {
    let key_controller = EventControllerKey::new();
    key_controller.set_propagation_phase(gtk::PropagationPhase::Capture);

    key_controller.connect_key_pressed({
        let state = state.clone();
        let components = components.clone();

        move |_, key, _code, modifier| {
            let (action, _mode, _is_active) = {
                let s = state.borrow();
                (s.shortcuts.get_action(key, modifier), s.mode, s.is_active)
            };

            if let Some(action) = action {
                debug!("Shortcut detected: {:?}", action);
                match action {
                    Action::Copy => {
                        perform_copy(&state, &components.window);
                        return glib::Propagation::Stop;
                    }
                    Action::Save => {
                        perform_save(state.clone(), components.window.clone());
                        return glib::Propagation::Stop;
                    }
                    Action::Undo => {
                        perform_undo(&state, &components.drawing.drawing_area);
                        return glib::Propagation::Stop;
                    }
                    Action::Cancel => {
                        let mut s = state.borrow_mut();
                        if s.is_active && s.mode == CaptureMode::Selection {
                            debug!("Canceling selection via shortcut");
                            s.exit_capture_mode();
                            components.window.unfullscreen();
                            components.header.header_bar.set_visible(true);
                            components.toolbar.tools_box.set_visible(s.final_image.is_some());
                            components.crop_toolbar.crop_tools_box.set_visible(false);
                            components.selection_toolbar.selection_tools_box.set_visible(false);
                            if s.final_image.is_none() {
                                components.drawing.placeholder_icon.set_visible(true);
                            }
                            drop(s);
                            components.drawing.drawing_area.queue_draw();
                            return glib::Propagation::Stop;
                        } else if s.is_crop_mode {
                            s.exit_crop_mode();
                            drop(s);
                            components.crop_toolbar.crop_tools_box.set_visible(false);
                            components.toolbar.tools_box.set_visible(true);
                            components.drawing.drawing_area.queue_draw();
                            return glib::Propagation::Stop;
                        }
                    }
                    Action::Confirm => {
                        let mut s = state.borrow_mut();
                        if s.is_active && s.mode == CaptureMode::Selection {
                            if confirm_selection(
                                &mut s,
                                &components.window,
                                &components.header.header_bar,
                                &components.toolbar.tools_box,
                                &components.crop_toolbar.crop_tools_box,
                            ) {
                                components.selection_toolbar.selection_tools_box.set_visible(false);
                                drop(s);
                                components.drawing.drawing_area.queue_draw();
                            }
                            return glib::Propagation::Stop;
                        }
                    }
                    Action::ToolPointer => {
                        let mut s = state.borrow_mut();
                        s.editor.set_tool(EditorTool::Pointer);
                        drop(s);
                        components.drawing.drawing_area.queue_draw();
                        return glib::Propagation::Stop;
                    }
                    Action::ToolPencil => {
                        let mut s = state.borrow_mut();
                        s.editor.set_tool(EditorTool::Pencil);
                        drop(s);
                        components.drawing.drawing_area.queue_draw();
                        return glib::Propagation::Stop;
                    }
                    Action::ToolRectangle => {
                        let mut s = state.borrow_mut();
                        s.editor.set_tool(EditorTool::Rectangle);
                        drop(s);
                        components.drawing.drawing_area.queue_draw();
                        return glib::Propagation::Stop;
                    }
                    Action::ToolText => {
                        let mut s = state.borrow_mut();
                        s.editor.set_tool(EditorTool::Text);
                        drop(s);
                        components.drawing.drawing_area.queue_draw();
                        return glib::Propagation::Stop;
                    }
                    Action::ToolCrop => {
                        let mut s = state.borrow_mut();
                        if s.final_image.is_some() {
                            s.is_crop_mode = true;
                            s.editor.set_tool(EditorTool::Crop);
                            components.toolbar.tools_box.set_visible(false);
                            components.crop_toolbar.crop_tools_box.set_visible(true);
                            drop(s);
                            components.drawing.drawing_area.queue_draw();
                            return glib::Propagation::Stop;
                        }
                    }
                    Action::SwitchToSelection => {
                        let mut s = state.borrow_mut();
                        s.mode = CaptureMode::Selection;
                        components.header.mode_selection_btn.set_active(true);
                        return glib::Propagation::Stop;
                    }
                    Action::SwitchToWindow => {
                        let mut s = state.borrow_mut();
                        s.mode = CaptureMode::Window;
                        components.header.mode_window_btn.set_active(true);
                        return glib::Propagation::Stop;
                    }
                    Action::SwitchToScreen => {
                        let mut s = state.borrow_mut();
                        s.mode = CaptureMode::Screen;
                        components.header.mode_screen_btn.set_active(true);
                        return glib::Propagation::Stop;
                    }
                    Action::TakeScreenshot => {
                        let mode = state.borrow().mode;
                        capture_screen_or_selection(&state, &components, mode);
                        return glib::Propagation::Stop;
                    }
                }
            }
            glib::Propagation::Proceed
        }
    });

    components.window.add_controller(key_controller);
}

pub fn capture_screen_or_selection(
    state: &Rc<RefCell<AppState>>,
    components: &UiComponents,
    mode: CaptureMode,
) {
    let window = &components.window;
    let header_bar = &components.header.header_bar;
    let tools_box = &components.toolbar.tools_box;
    let crop_tools_box = &components.crop_toolbar.crop_tools_box;
    let selection_tools_box = &components.selection_toolbar.selection_tools_box;
    let drawing_area = &components.drawing.drawing_area;
    let placeholder_icon = &components.drawing.placeholder_icon;

    if mode == CaptureMode::Window {
        show_window_selector(
            state,
            window,
            drawing_area,
            placeholder_icon,
            tools_box,
        );
        return;
    }

    window.set_visible(false);
    let context = gtk::glib::MainContext::default();
    while context.pending() {
        context.iteration(false);
    }
    std::thread::sleep(Duration::from_millis(200));

    match capture_primary_monitor() {
        Ok(result) => {
            let mut s = state.borrow_mut();
            s.original_screenshot = Some(result.pixbuf.clone());
            s.monitor_x = result.monitor_info.x;
            s.monitor_y = result.monitor_info.y;

            if mode == CaptureMode::Screen {
                s.final_image = Some(result.pixbuf);
                s.is_active = false;
                placeholder_icon.set_visible(false);
                tools_box.set_visible(true);
                window.set_visible(true);
            } else {
                s.is_active = true;
                s.mode = CaptureMode::Selection;
                s.final_image = Some(result.pixbuf);

                window.set_visible(true);
                window.fullscreen();
                header_bar.set_visible(false);
                tools_box.set_visible(false);
                crop_tools_box.set_visible(false);
                selection_tools_box.set_visible(true);
                placeholder_icon.set_visible(false);
            }
            drop(s);
            drawing_area.queue_draw();
        }
        Err(e) => {
            error!("Capture failed: {}", e);
            window.set_visible(true);
        }
    }
}

pub fn connect_all_handlers(state: &Rc<RefCell<AppState>>, components: &UiComponents) {
    connect_undo_handler(state, components);
    connect_copy_handler(state, components);
    connect_save_handler(state, components);
    connect_drag_handlers(state, components);
    connect_click_handlers(state, components);
    connect_crop_handlers(state, components);
    connect_selection_handlers(state, components);
    connect_screenshot_handler(state, components);
    connect_keyboard_handlers(state, components);

    let action_shortcuts = gio::SimpleAction::new("shortcuts", None);
    action_shortcuts.connect_activate({
        let state = state.clone();
        let window = components.window.clone();
        move |_, _| {
            shortcuts::show_shortcuts_dialog(&state, &window);
        }
    });
    components.window.add_action(&action_shortcuts);

    let action_about = gio::SimpleAction::new("about", None);
    action_about.connect_activate({
        let window = components.window.clone();
        move |_, _| {
            show_about_dialog(&window);
        }
    });
    components.window.add_action(&action_about);

    let menu_model = gio::Menu::new();
    menu_model.append(Some("Keyboard Shortcuts"), Some("win.shortcuts"));
    menu_model.append(Some("About Screenshot Tool"), Some("win.about"));
    components.header.menu_btn.set_menu_model(Some(&menu_model));
}
