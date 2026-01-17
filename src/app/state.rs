use gtk4 as gtk;

use crate::editor::EditorState;

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureMode {
    #[default]
    Selection,

    Window,

    Screen,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Selection {
    pub start_x: f64,
    pub start_y: f64,
    pub end_x: f64,
    pub end_y: f64,
}

impl Selection {
    pub fn new(start_x: f64, start_y: f64) -> Self {
        Self {
            start_x,
            start_y,
            end_x: start_x,
            end_y: start_y,
        }
    }

    pub fn update_end(&mut self, end_x: f64, end_y: f64) {
        self.end_x = end_x;
        self.end_y = end_y;
    }

    pub fn rectangle(&self) -> gtk::gdk::Rectangle {
        let x = self.start_x.min(self.end_x) as i32;
        let y = self.start_y.min(self.end_y) as i32;
        let w = (self.start_x - self.end_x).abs() as i32;
        let h = (self.start_y - self.end_y).abs() as i32;
        gtk::gdk::Rectangle::new(x, y, w, h)
    }

    pub fn is_significant(&self) -> bool {
        let rect = self.rectangle();
        rect.width() > 10 && rect.height() > 10
    }
}

pub struct AppState {
    pub mode: CaptureMode,

    pub original_screenshot: Option<gtk::gdk_pixbuf::Pixbuf>,

    pub final_image: Option<gtk::gdk_pixbuf::Pixbuf>,

    pub selection: Option<Selection>,

    pub is_active: bool,

    pub monitor_x: i32,

    pub monitor_y: i32,

    pub editor: EditorState,

    pub is_crop_mode: bool,

    pub delay_seconds: u32,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
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

    pub fn start_selection(&mut self, x: f64, y: f64) {
        self.selection = Some(Selection::new(x, y));
    }

    pub fn update_selection(&mut self, end_x: f64, end_y: f64) {
        if let Some(ref mut sel) = self.selection {
            sel.update_end(end_x, end_y);
        }
    }

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

    pub fn exit_capture_mode(&mut self) {
        self.is_active = false;
        self.selection = None;
        self.editor.reset();
    }

    pub fn exit_crop_mode(&mut self) {
        self.is_crop_mode = false;
        self.editor.tool_state.reset_drag();
    }

    pub fn increment_delay(&mut self) {
        if self.delay_seconds < 10 {
            self.delay_seconds += 1;
        }
    }

    pub fn decrement_delay(&mut self) {
        if self.delay_seconds > 0 {
            self.delay_seconds -= 1;
        }
    }
}
