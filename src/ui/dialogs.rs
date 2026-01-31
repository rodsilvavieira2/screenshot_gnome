use gtk4 as gtk;
use log::{debug, error, info};

use gtk::{Align, Orientation};
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::app::AppState;
use crate::capture::desktop::DesktopSession;
use crate::capture::window::{capture_window, list_capturable_windows, WindowInfo};

#[derive(Clone)]
pub struct TextPopoverComponents {
    pub text_popover: gtk::Popover,
    pub text_entry: gtk::Entry,
    pub text_confirm_btn: gtk::Button,
    pub text_cancel_btn: gtk::Button,
}

pub fn create_text_popover(drawing_area: &gtk::DrawingArea) -> TextPopoverComponents {
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
    text_popover.set_parent(drawing_area);

    TextPopoverComponents {
        text_popover,
        text_entry,
        text_confirm_btn,
        text_cancel_btn,
    }
}

pub fn connect_text_popover(
    state: &Rc<RefCell<AppState>>,
    drawing_area: &gtk::DrawingArea,
    components: &TextPopoverComponents,
) {
    debug!("Connecting text popover handlers");
    components.text_confirm_btn.connect_clicked({
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        let text_popover = components.text_popover.clone();
        let text_entry = components.text_entry.clone();
        move |_| {
            let text = text_entry.text().to_string();
            let mut s = state.borrow_mut();
            s.editor.commit_text(text);
            drop(s);
            text_popover.popdown();
            drawing_area.queue_draw();
        }
    });

    components.text_cancel_btn.connect_clicked({
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        let text_popover = components.text_popover.clone();
        move |_| {
            let mut s = state.borrow_mut();
            s.editor.cancel_text();
            drop(s);
            text_popover.popdown();
            drawing_area.queue_draw();
        }
    });

    components.text_entry.connect_activate({
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        let text_popover = components.text_popover.clone();
        let text_entry = components.text_entry.clone();
        move |_| {
            let text = text_entry.text().to_string();
            let mut s = state.borrow_mut();
            s.editor.commit_text(text);
            drop(s);
            text_popover.popdown();
            drawing_area.queue_draw();
        }
    });
}

pub fn show_window_selector(
    state: &Rc<RefCell<AppState>>,
    parent_window: &impl IsA<gtk::Window>,
    drawing_area: &gtk::DrawingArea,
    placeholder_icon: &gtk::Image,
    tools_box: &gtk::Box,
) {
    let window_selector = gtk::Window::builder()
        .title("Select Window")
        .modal(true)
        .transient_for(parent_window)
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

    let session = DesktopSession::detect();
    let session_label = gtk::Label::builder()
        .label(format!(
            "Session: {} • Backend: {}",
            session,
            session.window_list_backend()
        ))
        .css_classes(["dim-label"])
        .halign(Align::Start)
        .build();

    vbox.append(&gtk::Label::new(Some("Select a window to capture:")));
    vbox.append(&session_label);
    vbox.append(&scrolled_window);
    window_selector.set_child(Some(&vbox));

    let window_infos: Rc<RefCell<Vec<WindowInfo>>> = Rc::new(RefCell::new(Vec::new()));

    if let Ok(windows) = list_capturable_windows() {
        for win_info in windows {
            let row = gtk::Box::builder()
                .orientation(Orientation::Horizontal)
                .spacing(12)
                .build();

            let icon = gtk::Image::builder()
                .icon_name(win_info.icon_name_hint().to_lowercase())
                .pixel_size(32)
                .build();

            let label = gtk::Label::builder()
                .label(win_info.display_label())
                .halign(Align::Start)
                .ellipsize(gtk::pango::EllipsizeMode::End)
                .build();

            row.append(&icon);
            row.append(&label);

            list_box.append(&row);

            window_infos.borrow_mut().push(win_info);
        }
    }

    list_box.connect_row_activated({
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        let placeholder_icon = placeholder_icon.clone();
        let window_selector = window_selector.clone();
        let window_infos = window_infos.clone();
        let tools_box = tools_box.clone();
        move |_lb, row| {
            let idx = row.index();
            if idx >= 0 {
                let infos = window_infos.borrow();
                if let Some(window_info) = infos.get(idx as usize) {
                    match capture_window(window_info) {
                        Ok(result) => {
                            info!("Captured window: {}", result.window_info.debug_info());
                            let mut s = state.borrow_mut();
                            s.final_image = Some(result.pixbuf);
                            s.is_active = false;
                            s.editor.reset();

                            placeholder_icon.set_visible(false);
                            drawing_area.queue_draw();
                            tools_box.set_visible(true);
                            window_selector.close();
                        }
                        Err(e) => {
                            error!("Failed to capture window: {}", e);

                            let error_dialog = gtk::AlertDialog::builder()
                                .modal(true)
                                .message("Failed to Capture Window")
                                .detail(format!(
                                    "Could not capture the selected window.\n\nError: {}\n\n\
                                    Tip: Make sure the required screenshot tool is installed:\n\
                                    • Hyprland/Sway: grim\n\
                                    • GNOME: gnome-screenshot\n\
                                    • KDE: spectacle",
                                    e
                                ))
                                .buttons(["OK"])
                                .build();

                            error_dialog.show(Some(&window_selector));
                        }
                    }
                }
            }
        }
    });

    window_selector.present();
}
