//! Application state types
//!
//! This module contains the core state types for the screenshot application.

#![allow(dead_code)]

use gtk4 as gtk;

use crate::editor::EditorState;

/// The capture mode - how to capture the screenshot
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureMode {
    /// Capture a rectangular selection
    #[default]
    Selection,
    /// Capture a specific window
    Window,
    /// Capture the entire screen
    Screen,
}

/// A rectangular selection during capture
#[derive(Default, Clone, Copy, Debug)]
pub struct Selection {
    pub start_x: f64,
    pub start_y: f64,
    pub end_x: f64,
    pub end_y: f64,
}

impl Selection {
    /// Create a new selection with the given start point
    pub fn new(start_x: f64, start_y: f64) -> Self {
        Self {
            start_x,
            start_y,
            end_x: start_x,
            end_y: start_y,
        }
    }

    /// Update the end point of the selection
    pub fn update_end(&mut self, end_x: f64, end_y: f64) {
        self.end_x = end_x;
        self.end_y = end_y;
    }

    /// Get the selection as a normalized rectangle (positive width/height)
    pub fn rectangle(&self) -> gtk::gdk::Rectangle {
        let x = self.start_x.min(self.end_x) as i32;
        let y = self.start_y.min(self.end_y) as i32;
        let w = (self.start_x - self.end_x).abs() as i32;
        let h = (self.start_y - self.end_y).abs() as i32;
        gtk::gdk::Rectangle::new(x, y, w, h)
    }

    /// Check if the selection has a meaningful size
    pub fn is_significant(&self) -> bool {
        let rect = self.rectangle();
        rect.width() > 10 && rect.height() > 10
    }
}

/// Main application state
pub struct AppState {
    /// Current capture mode
    pub mode: CaptureMode,
    /// The original full screenshot (before cropping)
    pub original_screenshot: Option<gtk::gdk_pixbuf::Pixbuf>,
    /// The final image after cropping/editing
    pub final_image: Option<gtk::gdk_pixbuf::Pixbuf>,
    /// Current selection rectangle (during selection mode capture)
    pub selection: Option<Selection>,
    /// Whether capture overlay is active (for selection mode)
    pub is_active: bool,
    /// Monitor X offset (for multi-monitor support)
    pub monitor_x: i32,
    /// Monitor Y offset (for multi-monitor support)
    pub monitor_y: i32,
    /// Editor state (annotations, tools, etc.)
    pub editor: EditorState,
    /// Whether crop mode is active in the editor
    pub is_crop_mode: bool,
    /// Screenshot delay in seconds
    pub delay_seconds: u32,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    /// Create a new application state with default values
    pub fn new() -> Self {
        Self {
            mode: CaptureMode::Selection,
            original_screenshot: None,
            final_image: None,
            selection: None,
            is_active: false,
            monitor_x: 0,
            monitor_y: 0,
            editor: EditorState::new(),
            is_crop_mode: false,
            delay_seconds: 0,
        }
    }

    /// Reset the state for a new capture
    pub fn reset_for_capture(&mut self) {
        self.selection = None;
        self.is_active = false;
        self.is_crop_mode = false;
        self.editor.reset();
    }

    /// Start a new selection at the given point
    pub fn start_selection(&mut self, x: f64, y: f64) {
        self.selection = Some(Selection::new(x, y));
    }

    /// Update the current selection end point
    pub fn update_selection(&mut self, end_x: f64, end_y: f64) {
        if let Some(ref mut sel) = self.selection {
            sel.update_end(end_x, end_y);
        }
    }

    /// Check if there's a valid image to edit
    pub fn has_image(&self) -> bool {
        self.final_image.is_some()
    }

    /// Get the current image for display
    pub fn current_display_image(&self) -> Option<&gtk::gdk_pixbuf::Pixbuf> {
        if self.is_active {
            self.original_screenshot.as_ref()
        } else {
            self.final_image.as_ref()
        }
    }

    /// Apply crop to the original screenshot and store as final image
    pub fn apply_selection_crop(&mut self) -> bool {
        if let Some(sel) = self.selection {
            if sel.is_significant() {
                if let Some(ref orig) = self.original_screenshot {
                    let rect = sel.rectangle();
                    let crop_x = rect.x().max(0);
                    let crop_y = rect.y().max(0);
                    let crop_w = rect.width().min(orig.width() - crop_x);
                    let crop_h = rect.height().min(orig.height() - crop_y);

                    if crop_w > 0 && crop_h > 0 {
                        let cropped = orig.new_subpixbuf(crop_x, crop_y, crop_w, crop_h);
                        self.final_image = Some(cropped);
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Apply crop from the editor tool state
    pub fn apply_editor_crop(&mut self) -> bool {
        if let Some((x, y, w, h)) = self.editor.tool_state.get_drag_rect() {
            if w > 10.0 && h > 10.0 {
                if let Some(ref pixbuf) = self.final_image.clone() {
                    let crop_x = (x as i32).max(0);
                    let crop_y = (y as i32).max(0);
                    let crop_w = (w as i32).min(pixbuf.width() - crop_x);
                    let crop_h = (h as i32).min(pixbuf.height() - crop_y);

                    if crop_w > 0 && crop_h > 0 {
                        let cropped = pixbuf.new_subpixbuf(crop_x, crop_y, crop_w, crop_h);
                        self.final_image = Some(cropped);
                        self.editor.clear_annotations();
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Exit capture selection mode
    pub fn exit_capture_mode(&mut self) {
        self.is_active = false;
        self.selection = None;
        self.editor.reset();
    }

    /// Exit editor crop mode
    pub fn exit_crop_mode(&mut self) {
        self.is_crop_mode = false;
        self.editor.tool_state.reset_drag();
    }

    /// Increment the delay (max 10 seconds)
    pub fn increment_delay(&mut self) {
        if self.delay_seconds < 10 {
            self.delay_seconds += 1;
        }
    }

    /// Decrement the delay (min 0 seconds)
    pub fn decrement_delay(&mut self) {
        if self.delay_seconds > 0 {
            self.delay_seconds -= 1;
        }
    }
}
