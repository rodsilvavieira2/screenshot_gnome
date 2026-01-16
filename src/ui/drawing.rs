use gtk4 as gtk;

use gtk::DrawingArea;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::app::{AppState, CaptureMode};

pub struct DrawingComponents {
    pub drawing_area: DrawingArea,
    pub placeholder_icon: gtk::Image,
    pub picked_color_label: gtk::Label,
}

pub fn create_drawing_area(state: &Rc<RefCell<AppState>>) -> DrawingComponents {
    let drawing_area = DrawingArea::builder().hexpand(true).vexpand(true).build();

    setup_draw_function(&drawing_area, state);

    let placeholder_icon = gtk::Image::builder()
        .icon_name("image-x-generic-symbolic")
        .pixel_size(128)
        .opacity(0.2)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .build();

    let picked_color_label = gtk::Label::builder()
        .label("")
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Start)
        .margin_top(12)
        .visible(false)
        .build();
    picked_color_label.add_css_class("osd");

    DrawingComponents {
        drawing_area,
        placeholder_icon,
        picked_color_label,
    }
}

fn setup_draw_function(drawing_area: &DrawingArea, state: &Rc<RefCell<AppState>>) {
    drawing_area.set_draw_func({
        let state = state.clone();
        move |_, cr, width, height| {
            draw_content(&state, cr, width, height);
        }
    });
}

fn draw_content(state: &Rc<RefCell<AppState>>, cr: &gtk::cairo::Context, width: i32, height: i32) {
    let mut state = state.borrow_mut();
    let da_width = width as f64;
    let da_height = height as f64;

    cr.set_source_rgb(0.14, 0.14, 0.14);
    cr.paint().expect("Invalid cairo surface state");

    let pixbuf_opt = if state.is_active {
        state.original_screenshot.clone()
    } else {
        state.final_image.clone()
    };

    if let Some(pixbuf) = pixbuf_opt {
        let img_width = pixbuf.width() as f64;
        let img_height = pixbuf.height() as f64;

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

        state
            .editor
            .update_display_transform(scale, offset_x, offset_y);

        cr.save().expect("Failed to save cairo context");
        cr.translate(offset_x, offset_y);
        cr.scale(scale, scale);
        cr.set_source_pixbuf(&pixbuf, 0.0, 0.0);
        cr.paint().expect("Failed to paint pixbuf");
        cr.restore().expect("Failed to restore cairo context");

        if state.is_active && state.mode == CaptureMode::Selection {
            draw_selection_overlay(&state, cr, da_width, da_height);
        }

        if state.is_crop_mode {
            draw_crop_overlay(&state, cr, da_width, da_height, scale);
        }

        if !state.is_active {
            state.editor.draw_annotations(cr);
        }

        draw_pending_text_cursor(&state, cr);
    }
}

fn draw_selection_overlay(
    state: &AppState,
    cr: &gtk::cairo::Context,
    da_width: f64,
    da_height: f64,
) {
    if let Some(sel) = state.selection {
        let rect = sel.rectangle();
        let rx = rect.x() as f64;
        let ry = rect.y() as f64;
        let rw = rect.width() as f64;
        let rh = rect.height() as f64;

        cr.set_source_rgba(0.0, 0.0, 0.0, 0.5);

        cr.rectangle(0.0, 0.0, da_width, ry);

        cr.rectangle(0.0, ry + rh, da_width, da_height - (ry + rh));

        cr.rectangle(0.0, ry, rx, rh);

        cr.rectangle(rx + rw, ry, da_width - (rx + rw), rh);
        cr.fill().expect("Failed to fill dimming rects");

        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.set_line_width(2.0);
        cr.rectangle(rx, ry, rw, rh);
        cr.stroke().expect("Failed to stroke selection border");
    }
}

fn draw_crop_overlay(
    state: &AppState,
    cr: &gtk::cairo::Context,
    da_width: f64,
    da_height: f64,
    scale: f64,
) {
    if let Some((x, y, w, h)) = state.editor.tool_state.get_drag_rect() {
        let (dx, dy) = state.editor.image_to_display_coords(x, y);
        let dw = w * scale;
        let dh = h * scale;

        cr.set_source_rgba(0.0, 0.0, 0.0, 0.5);

        cr.rectangle(0.0, 0.0, da_width, dy);

        cr.rectangle(0.0, dy + dh, da_width, da_height - (dy + dh));

        cr.rectangle(0.0, dy, dx, dh);

        cr.rectangle(dx + dw, dy, da_width - (dx + dw), dh);
        let _ = cr.fill();

        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.set_line_width(2.0);
        cr.rectangle(dx, dy, dw, dh);
        let _ = cr.stroke();
    }
}

fn draw_pending_text_cursor(state: &AppState, cr: &gtk::cairo::Context) {
    if let Some(ref pending) = state.editor.pending_text {
        let (dx, dy) = state.editor.image_to_display_coords(pending.x, pending.y);
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.8);
        cr.set_line_width(2.0);
        cr.move_to(dx, dy - 20.0);
        cr.line_to(dx, dy + 5.0);
        let _ = cr.stroke();
    }
}