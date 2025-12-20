#![allow(dead_code)]

use gtk4 as gtk;

use gtk::{Align, Orientation};
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::app::AppState;
use crate::editor::EditorTool;

pub struct ToolbarComponents {
    pub tools_box: gtk::Box,
    pub tool_pointer_btn: gtk::ToggleButton,
    pub tool_pencil_btn: gtk::ToggleButton,
    pub tool_rectangle_btn: gtk::ToggleButton,
    pub tool_crop_btn: gtk::ToggleButton,
    pub tool_text_btn: gtk::ToggleButton,
    pub color_button: gtk::ColorDialogButton,
    pub color_picker_circle: gtk::DrawingArea,
    pub undo_btn: gtk::Button,
    pub copy_btn: gtk::Button,
    pub save_btn: gtk::Button,
}

pub struct CropToolbarComponents {
    pub crop_tools_box: gtk::Box,
    pub confirm_btn: gtk::Button,
    pub cancel_btn: gtk::Button,
}

pub fn create_toolbar(state: &Rc<RefCell<AppState>>) -> ToolbarComponents {
    let color_button = gtk::ColorDialogButton::builder()
        .dialog(&gtk::ColorDialog::new())
        .rgba(&gtk::gdk::RGBA::new(1.0, 0.0, 0.0, 1.0))
        .tooltip_text("Select Color")
        .build();

    let color_picker_circle = create_color_picker_circle(state);

    connect_color_button(state, &color_button, &color_picker_circle);

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

    let undo_btn = gtk::Button::builder()
        .icon_name("edit-undo-symbolic")
        .tooltip_text("Undo")
        .build();
    undo_btn.add_css_class("flat");

    let copy_btn = gtk::Button::builder()
        .icon_name("edit-copy-symbolic")
        .tooltip_text("Copy to Clipboard")
        .build();
    copy_btn.add_css_class("flat");

    let save_btn = gtk::Button::builder()
        .icon_name("document-save-symbolic")
        .tooltip_text("Save")
        .build();
    save_btn.add_css_class("suggested-action");

    let tools_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .halign(Align::Center)
        .valign(Align::End)
        .margin_bottom(24)
        .build();
    tools_box.add_css_class("osd");
    tools_box.add_css_class("toolbar");

    tools_box.append(&tool_buttons_box);
    tools_box.append(&undo_btn);
    tools_box.append(&copy_btn);
    tools_box.append(&save_btn);

    ToolbarComponents {
        tools_box,
        tool_pointer_btn,
        tool_pencil_btn,
        tool_rectangle_btn,
        tool_crop_btn,
        tool_text_btn,
        color_button,
        color_picker_circle,
        undo_btn,
        copy_btn,
        save_btn,
    }
}

pub fn create_crop_toolbar() -> CropToolbarComponents {
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

    CropToolbarComponents {
        crop_tools_box,
        confirm_btn,
        cancel_btn,
    }
}

fn create_color_picker_circle(state: &Rc<RefCell<AppState>>) -> gtk::DrawingArea {
    let color_picker_circle = gtk::DrawingArea::builder()
        .width_request(20)
        .height_request(20)
        .build();

    color_picker_circle.set_draw_func({
        let state = state.clone();
        move |_, cr, width, height| {
            let state = state.borrow();
            let color = state.editor.current_color();

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

    color_picker_circle
}

fn connect_color_button(
    state: &Rc<RefCell<AppState>>,
    color_button: &gtk::ColorDialogButton,
    color_picker_circle: &gtk::DrawingArea,
) {
    color_button.connect_rgba_notify({
        let state = state.clone();
        let color_picker_circle = color_picker_circle.clone();
        move |btn| {
            let color = btn.rgba();
            state.borrow_mut().editor.set_color(color);
            color_picker_circle.queue_draw();
        }
    });
}

pub fn connect_tool_buttons(
    state: &Rc<RefCell<AppState>>,
    components: &ToolbarComponents,
    tools_box: &gtk::Box,
    crop_tools_box: &gtk::Box,
) {
    components.tool_pointer_btn.connect_toggled({
        let state = state.clone();
        move |btn| {
            if btn.is_active() {
                let mut s = state.borrow_mut();
                s.editor.set_tool(EditorTool::Pointer);
                s.is_crop_mode = false;
            }
        }
    });

    components.tool_pencil_btn.connect_toggled({
        let state = state.clone();
        move |btn| {
            if btn.is_active() {
                let mut s = state.borrow_mut();
                s.editor.set_tool(EditorTool::Pencil);
                s.is_crop_mode = false;
            }
        }
    });

    components.tool_rectangle_btn.connect_toggled({
        let state = state.clone();
        move |btn| {
            if btn.is_active() {
                let mut s = state.borrow_mut();
                s.editor.set_tool(EditorTool::Rectangle);
                s.is_crop_mode = false;
            }
        }
    });

    components.tool_crop_btn.connect_toggled({
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

    components.tool_text_btn.connect_toggled({
        let state = state.clone();
        move |btn| {
            if btn.is_active() {
                let mut s = state.borrow_mut();
                s.editor.set_tool(EditorTool::Text);
                s.is_crop_mode = false;
            }
        }
    });
}
