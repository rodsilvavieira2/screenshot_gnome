//! Editor module for screenshot annotation and editing tools
//!
//! This module provides tools for:
//! - Drawing rectangles
//! - Free-hand drawing
//! - Adding text annotations
//! - Cropping images
//! - Picking colors from images
//! - Clipboard operations

#![allow(dead_code)]

pub mod annotations;
pub mod clipboard;
pub mod color_picker;
pub mod tools;

// Re-export commonly used types
pub use annotations::{
    Annotation, AnnotationList, FreeDrawAnnotation, RectangleAnnotation, TextAnnotation,
};
pub use clipboard::ClipboardManager;
pub use color_picker::{ColorPickerState, pick_color_from_pixbuf};
pub use tools::{EditorTool, ToolState};

use gtk4::gdk::RGBA;
use gtk4::gdk_pixbuf::Pixbuf;

/// Main editor state that combines all editing functionality
#[derive(Clone, Debug)]
pub struct EditorState {
    /// Tool state (active tool, color, line width, etc.)
    pub tool_state: ToolState,
    /// All annotations on the current image
    pub annotations: AnnotationList,
    /// Color picker state
    pub color_picker: ColorPickerState,
    /// Text input state (for text tool)
    pub pending_text: Option<PendingText>,
    /// Whether editing mode is active
    pub is_editing: bool,
    /// Image display scaling info (for coordinate conversion)
    pub display_scale: f64,
    pub display_offset_x: f64,
    pub display_offset_y: f64,
}

/// Pending text annotation being edited
#[derive(Clone, Debug)]
pub struct PendingText {
    pub x: f64,
    pub y: f64,
    pub text: String,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            tool_state: ToolState::default(),
            annotations: AnnotationList::new(),
            color_picker: ColorPickerState::new(),
            pending_text: None,
            is_editing: false,
            display_scale: 1.0,
            display_offset_x: 0.0,
            display_offset_y: 0.0,
        }
    }
}

impl EditorState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the current active tool
    pub fn set_tool(&mut self, tool: EditorTool) {
        self.tool_state.set_tool(tool);
        self.pending_text = None;
    }

    /// Get the current active tool
    pub fn current_tool(&self) -> EditorTool {
        self.tool_state.active_tool
    }

    /// Set the drawing color
    pub fn set_color(&mut self, color: RGBA) {
        self.tool_state.set_color(color);
    }

    /// Get the current drawing color
    pub fn current_color(&self) -> RGBA {
        self.tool_state.color
    }

    /// Update display transformation info (for coordinate conversion)
    pub fn update_display_transform(&mut self, scale: f64, offset_x: f64, offset_y: f64) {
        self.display_scale = scale;
        self.display_offset_x = offset_x;
        self.display_offset_y = offset_y;
    }

    /// Convert display coordinates to image coordinates
    pub fn display_to_image_coords(&self, display_x: f64, display_y: f64) -> (f64, f64) {
        let img_x = (display_x - self.display_offset_x) / self.display_scale;
        let img_y = (display_y - self.display_offset_y) / self.display_scale;
        (img_x, img_y)
    }

    /// Convert image coordinates to display coordinates
    pub fn image_to_display_coords(&self, img_x: f64, img_y: f64) -> (f64, f64) {
        let display_x = img_x * self.display_scale + self.display_offset_x;
        let display_y = img_y * self.display_scale + self.display_offset_y;
        (display_x, display_y)
    }

    /// Handle drag start event
    pub fn on_drag_start(&mut self, x: f64, y: f64) {
        let (img_x, img_y) = self.display_to_image_coords(x, y);
        self.tool_state.start_drag(img_x, img_y);

        match self.tool_state.active_tool {
            EditorTool::Pencil => {
                let mut free_draw =
                    FreeDrawAnnotation::new(self.tool_state.color, self.tool_state.line_width);
                free_draw.add_point(img_x, img_y);
                self.annotations
                    .set_current(Some(Annotation::FreeDraw(free_draw)));
            }
            EditorTool::Rectangle => {
                // Rectangle will be created during drag update
            }
            EditorTool::Text => {
                self.pending_text = Some(PendingText {
                    x: img_x,
                    y: img_y,
                    text: String::new(),
                });
            }
            _ => {}
        }
    }

    /// Handle drag update event
    pub fn on_drag_update(&mut self, x: f64, y: f64) {
        let (img_x, img_y) = self.display_to_image_coords(x, y);
        self.tool_state.update_drag(img_x, img_y);

        match self.tool_state.active_tool {
            EditorTool::Pencil => {
                if let Some(Annotation::FreeDraw(ref mut draw)) =
                    self.annotations.current().cloned()
                {
                    let mut draw = draw.clone();
                    draw.add_point(img_x, img_y);
                    self.annotations
                        .set_current(Some(Annotation::FreeDraw(draw)));
                }
            }
            EditorTool::Rectangle => {
                if let (Some((start_x, start_y)), Some((end_x, end_y))) =
                    (self.tool_state.drag_start, self.tool_state.drag_current)
                {
                    let rect = RectangleAnnotation::from_corners(
                        start_x,
                        start_y,
                        end_x,
                        end_y,
                        self.tool_state.color,
                        self.tool_state.line_width,
                    );
                    self.annotations
                        .set_current(Some(Annotation::Rectangle(rect)));
                }
            }
            _ => {}
        }
    }

    /// Handle drag end event
    pub fn on_drag_end(&mut self, _x: f64, _y: f64) {
        match self.tool_state.active_tool {
            EditorTool::Pencil | EditorTool::Rectangle => {
                self.annotations.commit_current();
            }
            _ => {}
        }
        self.tool_state.end_drag();
    }

    /// Handle click event (for color picker and text placement)
    pub fn on_click(&mut self, x: f64, y: f64, pixbuf: Option<&Pixbuf>) -> Option<RGBA> {
        let (img_x, img_y) = self.display_to_image_coords(x, y);

        match self.tool_state.active_tool {
            EditorTool::ColorPicker => {
                if let Some(pb) = pixbuf {
                    if let Ok(picked) = pick_color_from_pixbuf(pb, img_x as i32, img_y as i32) {
                        let color = picked.color;
                        self.color_picker.set_picked_color(picked);
                        self.tool_state.set_color(color);
                        return Some(color);
                    }
                }
            }
            EditorTool::Text => {
                self.pending_text = Some(PendingText {
                    x: img_x,
                    y: img_y,
                    text: String::new(),
                });
            }
            _ => {}
        }
        None
    }

    /// Commit pending text annotation
    pub fn commit_text(&mut self, text: String) {
        if let Some(pending) = self.pending_text.take() {
            if !text.is_empty() {
                let text_annotation = TextAnnotation::new(
                    pending.x,
                    pending.y,
                    text,
                    self.tool_state.color,
                    self.tool_state.font_size,
                );
                self.annotations.add(Annotation::Text(text_annotation));
            }
        }
    }

    /// Cancel pending text
    pub fn cancel_text(&mut self) {
        self.pending_text = None;
    }

    /// Undo the last annotation
    pub fn undo(&mut self) -> bool {
        self.annotations.undo()
    }

    /// Clear all annotations
    pub fn clear_annotations(&mut self) {
        self.annotations.clear();
    }

    /// Draw all annotations on a cairo context
    pub fn draw_annotations(&self, cr: &gtk4::cairo::Context) {
        self.annotations.draw_all(
            cr,
            self.display_scale,
            self.display_offset_x,
            self.display_offset_y,
        );
    }

    /// Reset editor state for a new image
    pub fn reset(&mut self) {
        self.annotations.clear();
        self.color_picker.clear();
        self.pending_text = None;
        self.tool_state.reset_drag();
    }
}
