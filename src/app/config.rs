use gtk::gdk;
use gtk4 as gtk;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    Copy,
    Save,
    Undo,
    Cancel,
    Confirm,
    ToolPointer,
    ToolPencil,
    ToolRectangle,
    ToolText,
    ToolCrop,
    SwitchToSelection,
    SwitchToWindow,
    SwitchToScreen,
}

impl Action {
    #[allow(dead_code)]
    pub fn label(&self) -> &str {
        match self {
            Action::Copy => "Copy to Clipboard",
            Action::Save => "Save to File",
            Action::Undo => "Undo",
            Action::Cancel => "Cancel / Exit",
            Action::Confirm => "Confirm Selection",
            Action::ToolPointer => "Select Pointer Tool",
            Action::ToolPencil => "Select Pencil Tool",
            Action::ToolRectangle => "Select Rectangle Tool",
            Action::ToolText => "Select Text Tool",
            Action::ToolCrop => "Select Crop Tool",
            Action::SwitchToSelection => "Switch to Selection Mode",
            Action::SwitchToWindow => "Switch to Window Mode",
            Action::SwitchToScreen => "Switch to Screen Mode",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Shortcut {
    pub key: gdk::Key,
    pub modifiers: gdk::ModifierType,
}

#[derive(Debug, Clone)]
pub struct ShortcutConfig {
    bindings: HashMap<Action, Shortcut>,
}

impl Default for ShortcutConfig {
    fn default() -> Self {
        let mut bindings = HashMap::new();

        // Standard Actions
        bindings.insert(
            Action::Copy,
            Shortcut {
                key: gdk::Key::c,
                modifiers: gdk::ModifierType::CONTROL_MASK,
            },
        );
        bindings.insert(
            Action::Save,
            Shortcut {
                key: gdk::Key::s,
                modifiers: gdk::ModifierType::CONTROL_MASK,
            },
        );
        bindings.insert(
            Action::Undo,
            Shortcut {
                key: gdk::Key::z,
                modifiers: gdk::ModifierType::CONTROL_MASK,
            },
        );
        bindings.insert(
            Action::Cancel,
            Shortcut {
                key: gdk::Key::Escape,
                modifiers: gdk::ModifierType::empty(),
            },
        );
        bindings.insert(
            Action::Confirm,
            Shortcut {
                key: gdk::Key::Return,
                modifiers: gdk::ModifierType::empty(),
            },
        );

        // Tool Switching
        bindings.insert(
            Action::ToolPointer,
            Shortcut {
                key: gdk::Key::v,
                modifiers: gdk::ModifierType::empty(),
            },
        );
        bindings.insert(
            Action::ToolPencil,
            Shortcut {
                key: gdk::Key::p,
                modifiers: gdk::ModifierType::empty(),
            },
        );
        bindings.insert(
            Action::ToolRectangle,
            Shortcut {
                key: gdk::Key::r,
                modifiers: gdk::ModifierType::empty(),
            },
        );
        bindings.insert(
            Action::ToolText,
            Shortcut {
                key: gdk::Key::t,
                modifiers: gdk::ModifierType::empty(),
            },
        );
        bindings.insert(
            Action::ToolCrop,
            Shortcut {
                key: gdk::Key::c,
                modifiers: gdk::ModifierType::empty(),
            },
        );

        // Mode Switching
        bindings.insert(
            Action::SwitchToSelection,
            Shortcut {
                key: gdk::Key::s,
                modifiers: gdk::ModifierType::ALT_MASK,
            },
        );
        bindings.insert(
            Action::SwitchToWindow,
            Shortcut {
                key: gdk::Key::w,
                modifiers: gdk::ModifierType::ALT_MASK,
            },
        );
        bindings.insert(
            Action::SwitchToScreen,
            Shortcut {
                key: gdk::Key::d,
                modifiers: gdk::ModifierType::ALT_MASK,
            },
        );

        Self { bindings }
    }
}

impl ShortcutConfig {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_action(&self, key: gdk::Key, modifiers: gdk::ModifierType) -> Option<Action> {
        // Filter out irrelevant modifiers like NumLock/CapsLock/ScrollLock
        let mask = gdk::ModifierType::CONTROL_MASK
            | gdk::ModifierType::SHIFT_MASK
            | gdk::ModifierType::ALT_MASK
            | gdk::ModifierType::SUPER_MASK
            | gdk::ModifierType::META_MASK;

        let clean_mods = modifiers & mask;

        for (action, shortcut) in &self.bindings {
            if shortcut.key == key && shortcut.modifiers == clean_mods {
                return Some(*action);
            }

            // Handle Keypad Enter as alias for Return
            if *action == Action::Confirm
                && key == gdk::Key::KP_Enter
                && shortcut.key == gdk::Key::Return
                && shortcut.modifiers == clean_mods
            {
                return Some(*action);
            }
        }
        None
    }

    pub fn get_shortcut_label(&self, action: Action) -> String {
        if let Some(sc) = self.bindings.get(&action) {
            return gtk::accelerator_name(sc.key, sc.modifiers).to_string();
        }
        String::new()
    }

    #[allow(dead_code)]
    pub fn set_shortcut(&mut self, action: Action, key: gdk::Key, modifiers: gdk::ModifierType) {
        self.bindings.insert(action, Shortcut { key, modifiers });
    }

    #[allow(dead_code)]
    pub fn get_all_shortcuts(&self) -> &HashMap<Action, Shortcut> {
        &self.bindings
    }
}
