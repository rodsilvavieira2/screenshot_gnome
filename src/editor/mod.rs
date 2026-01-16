#![allow(dead_code)]

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
use gtk4::gdk_pixbuf::Pixbuf;

#[derive(Clone, Debug)]
pub struct EditorState {
    pub tool_state: ToolState,

    pub annotations: AnnotationList,

    pub color_picker: ColorPickerState,

    pub pending_text: Option<PendingText>,

    pub is_editing: bool,

    pub last_drag_moved: bool,

    pub display_scale: f64,
    pub display_offset_x: f64,
    pub display_offset_y: f64,
}

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
        self.tool_state.set_tool(tool);
        self.pending_text = None;
    }

    pub fn current_tool(&self) -> EditorTool {
        self.tool_state.active_tool
    }

    pub fn set_color(&mut self, color: RGBA) {
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

    pub fn on_drag_start(&mut self, x: f64, y: f64) {
        self.last_drag_moved = false;
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
            EditorTool::Rectangle => {}
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

    pub fn on_drag_end(&mut self, _x: f64, _y: f64) {
        match self.tool_state.active_tool {
            EditorTool::Pencil | EditorTool::Rectangle => {
                self.annotations.commit_current();
            }
            _ => {}
        }
        self.tool_state.end_drag();
    }

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
        self.annotations.clear();
        self.color_picker.clear();
        self.pending_text = None;
        self.tool_state.reset_drag();
    }

    pub fn pointer_drag_start(&mut self, display_x: f64, display_y: f64) -> bool {
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
        self.last_drag_moved = self.tool_state.moved_annotation;
        self.tool_state.end_annotation_drag();
    }

    pub fn is_pointer_dragging(&self) -> bool {
        self.tool_state.is_dragging_annotation
    }

    pub fn did_move_annotation(&self) -> bool {
        self.tool_state.moved_annotation
    }

    pub fn deselect_annotation(&mut self) {
        self.annotations.deselect();
    }

    pub fn selected_annotation_index(&self) -> Option<usize> {
        self.annotations.selected_index()
    }
}
