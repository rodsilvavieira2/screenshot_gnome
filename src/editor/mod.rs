pub mod annotations;
pub mod clipboard;
pub mod color_picker;
pub mod tools;

pub use annotations::{
    Annotation, AnnotationList, FreeDrawAnnotation, RectangleAnnotation, TextAnnotation,
};
pub use clipboard::ClipboardManager;
pub use color_picker::{pick_color_from_pixbuf, ColorPickerState};
pub use tools::{EditorTool, ToolState};

use gtk4::gdk::RGBA;
use log::debug;

#[derive(Clone, Debug)]
pub struct EditorState {
    pub tool_state: ToolState,

    pub annotations: AnnotationList,

    pub color_picker: ColorPickerState,

    pub pending_text: Option<PendingText>,

    pub last_drag_moved: bool,

    pub display_scale: f64,
    pub display_offset_x: f64,
    pub display_offset_y: f64,
}

#[derive(Clone, Debug)]
pub struct PendingText {
    pub x: f64,
    pub y: f64,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            tool_state: ToolState::default(),
            annotations: AnnotationList::new(),
            color_picker: ColorPickerState::new(),
            pending_text: None,
            last_drag_moved: false,
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

    pub fn set_tool(&mut self, tool: EditorTool) {
        debug!("Setting tool to {:?}", tool);
        self.tool_state.set_tool(tool);
        self.pending_text = None;
    }

    pub fn current_tool(&self) -> EditorTool {
        self.tool_state.active_tool
    }

    pub fn set_color(&mut self, color: RGBA) {
        debug!("Setting color to {:?}", color);
        self.tool_state.set_color(color);
    }

    pub fn current_color(&self) -> RGBA {
        self.tool_state.color
    }

    pub fn update_display_transform(&mut self, scale: f64, offset_x: f64, offset_y: f64) {
        self.display_scale = scale;
        self.display_offset_x = offset_x;
        self.display_offset_y = offset_y;
    }

    pub fn display_to_image_coords(&self, display_x: f64, display_y: f64) -> (f64, f64) {
        let img_x = (display_x - self.display_offset_x) / self.display_scale;
        let img_y = (display_y - self.display_offset_y) / self.display_scale;
        (img_x, img_y)
    }

    pub fn image_to_display_coords(&self, img_x: f64, img_y: f64) -> (f64, f64) {
        let display_x = img_x * self.display_scale + self.display_offset_x;
        let display_y = img_y * self.display_scale + self.display_offset_y;
        (display_x, display_y)
    }

    pub fn commit_text(&mut self, text: String) {
        debug!("Committing text: {}", text);
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
                // Select the newly added text
                let new_index = self.annotations.len() - 1;
                self.annotations.set_selected(Some(new_index));
            }
        }
    }

    pub fn cancel_text(&mut self) {
        self.pending_text = None;
    }

    pub fn undo(&mut self) -> bool {
        debug!("Undo operation requested");
        self.annotations.undo()
    }

    pub fn clear_annotations(&mut self) {
        self.annotations.clear();
    }

    pub fn draw_annotations(&self, cr: &gtk4::cairo::Context) {
        self.annotations.draw_all(
            cr,
            self.display_scale,
            self.display_offset_x,
            self.display_offset_y,
        );
    }

    pub fn reset(&mut self) {
        debug!("Resetting editor state");
        self.annotations.clear();
        self.color_picker.clear();
        self.pending_text = None;
        self.tool_state.reset_drag();
    }

    pub fn pointer_drag_start(&mut self, display_x: f64, display_y: f64) -> bool {
        debug!("Pointer drag start at ({}, {})", display_x, display_y);
        self.last_drag_moved = false;
        let (img_x, img_y) = self.display_to_image_coords(display_x, display_y);

        if let Some(index) = self.annotations.hit_test(img_x, img_y) {
            self.annotations.set_selected(Some(index));

            if let Some((ann_x, ann_y)) = self.annotations.selected_position() {
                self.tool_state
                    .start_annotation_drag(img_x, img_y, ann_x, ann_y);
                return true;
            }
        } else {
            self.annotations.deselect();
        }

        false
    }

    pub fn pointer_drag_update(&mut self, display_x: f64, display_y: f64) {
        if !self.tool_state.is_dragging_annotation {
            return;
        }

        let (img_x, img_y) = self.display_to_image_coords(display_x, display_y);
        self.tool_state.update_annotation_drag(img_x, img_y);

        if let Some((offset_x, offset_y)) = self.tool_state.pointer_drag_offset {
            let new_x = img_x - offset_x;
            let new_y = img_y - offset_y;

            if let Some((old_x, old_y)) = self.annotations.selected_position() {
                let dx = new_x - old_x;
                let dy = new_y - old_y;
                self.annotations.move_selected(dx, dy);
            }
        }
    }

    pub fn pointer_drag_end(&mut self) {
        debug!("Pointer drag end");
        self.last_drag_moved = self.tool_state.moved_annotation;
        self.tool_state.end_annotation_drag();
    }
}
