use adw::prelude::*;
use gtk4 as gtk;
use libadwaita as adw;
use std::cell::RefCell;
use std::rc::Rc;

use crate::app::config::Action;
use crate::app::AppState;

pub fn show_shortcuts_dialog(state: &Rc<RefCell<AppState>>, parent: &impl IsA<gtk::Window>) {
    let window = adw::PreferencesWindow::builder()
        .transient_for(parent)
        .modal(true)
        .title("Keyboard Shortcuts")
        .default_width(500)
        .default_height(600)
        .build();

    let page = adw::PreferencesPage::new();
    window.add(&page);

    let group_general = adw::PreferencesGroup::builder().title("General").build();
    add_action_row(state, &group_general, Action::Copy, "Copy to Clipboard");
    add_action_row(state, &group_general, Action::Save, "Save to File");
    add_action_row(state, &group_general, Action::Undo, "Undo");
    add_action_row(state, &group_general, Action::Cancel, "Cancel / Exit");
    add_action_row(state, &group_general, Action::Confirm, "Confirm Selection");
    page.add(&group_general);

    let group_tools = adw::PreferencesGroup::builder().title("Tools").build();
    add_action_row(state, &group_tools, Action::ToolPointer, "Pointer");
    add_action_row(state, &group_tools, Action::ToolPencil, "Pencil");
    add_action_row(state, &group_tools, Action::ToolRectangle, "Rectangle");
    add_action_row(state, &group_tools, Action::ToolText, "Text");
    add_action_row(state, &group_tools, Action::ToolCrop, "Crop");
    page.add(&group_tools);

    let group_modes = adw::PreferencesGroup::builder()
        .title("Capture Modes")
        .build();
    add_action_row(
        state,
        &group_modes,
        Action::SwitchToSelection,
        "Selection Mode",
    );
    add_action_row(state, &group_modes, Action::SwitchToWindow, "Window Mode");
    add_action_row(state, &group_modes, Action::SwitchToScreen, "Screen Mode");
    page.add(&group_modes);

    window.present();
}

fn add_action_row(
    state: &Rc<RefCell<AppState>>,
    group: &adw::PreferencesGroup,
    action: Action,
    title: &str,
) {
    let s = state.borrow();
    let shortcut_label = s.shortcuts.get_shortcut_label(action);
    drop(s);

    let row = adw::ActionRow::builder().title(title).build();

    let shortcut_btn = gtk::Button::builder()
        .label(&shortcut_label)
        .valign(gtk::Align::Center)
        .css_classes(["flat"])
        .build();

    if shortcut_label.is_empty() {
        shortcut_btn.set_label("Disabled");
        shortcut_btn.add_css_class("dim-label");
    }

    // Future implementation: Shortcut recording popover
    shortcut_btn.set_sensitive(false);

    row.add_suffix(&shortcut_btn);
    group.add(&row);
}
