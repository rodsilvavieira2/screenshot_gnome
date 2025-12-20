#![allow(dead_code)]

use gtk4 as gtk;
use libadwaita as adw;

use adw::prelude::*;
use gtk::Orientation;
use std::cell::RefCell;
use std::rc::Rc;

use crate::app::{AppState, CaptureMode};

pub struct HeaderComponents {
    pub header_bar: adw::HeaderBar,
    pub take_screenshot_btn: gtk::Button,
    pub mode_selection: gtk::ToggleButton,
    pub mode_window: gtk::ToggleButton,
    pub mode_screen: gtk::ToggleButton,
    pub delay_value: gtk::Label,
    pub delay_minus: gtk::Button,
    pub delay_plus: gtk::Button,
}

pub fn create_header_bar(state: &Rc<RefCell<AppState>>) -> HeaderComponents {
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

    connect_mode_toggles(state, &mode_selection, &mode_window, &mode_screen);

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

    connect_delay_controls(state, &delay_value, &delay_minus, &delay_plus);

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

    HeaderComponents {
        header_bar,
        take_screenshot_btn,
        mode_selection,
        mode_window,
        mode_screen,
        delay_value,
        delay_minus,
        delay_plus,
    }
}

fn connect_mode_toggles(
    state: &Rc<RefCell<AppState>>,
    mode_selection: &gtk::ToggleButton,
    mode_window: &gtk::ToggleButton,
    mode_screen: &gtk::ToggleButton,
) {
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
}

fn connect_delay_controls(
    state: &Rc<RefCell<AppState>>,
    delay_value: &gtk::Label,
    delay_minus: &gtk::Button,
    delay_plus: &gtk::Button,
) {
    delay_minus.connect_clicked({
        let state = state.clone();
        let delay_value = delay_value.clone();
        move |_| {
            let mut s = state.borrow_mut();
            s.decrement_delay();
            delay_value.set_label(&s.delay_seconds.to_string());
        }
    });

    delay_plus.connect_clicked({
        let state = state.clone();
        let delay_value = delay_value.clone();
        move |_| {
            let mut s = state.borrow_mut();
            s.increment_delay();
            delay_value.set_label(&s.delay_seconds.to_string());
        }
    });
}
