use gtk4::gdk::RGBA;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EditorTool {
    #[default]
    Pointer,
    Pencil,
    Rectangle,
    Crop,
    Text,
    ColorPicker,
}

impl EditorTool {
    pub fn icon_name(&self) -> &'static str {
        match self {
            EditorTool::Pointer => "input-mouse-symbolic",
            EditorTool::Pencil => "document-edit-symbolic",
            EditorTool::Rectangle => "media-playback-stop-symbolic",
            EditorTool::Crop => "crop-symbolic",
            EditorTool::Text => "insert-text-symbolic",
            EditorTool::ColorPicker => "color-select-symbolic",
        }
    }

    pub fn tooltip(&self) -> &'static str {
        match self {
            EditorTool::Pointer => "Pointer",
            EditorTool::Pencil => "Free Draw",
            EditorTool::Rectangle => "Rectangle",
            EditorTool::Crop => "Crop",
            EditorTool::Text => "Add Text",
            EditorTool::ColorPicker => "Pick Color",
        }
    }

    pub fn all() -> &'static [EditorTool] {
        &[
            EditorTool::Pointer,
            EditorTool::Pencil,
            EditorTool::Rectangle,
            EditorTool::Crop,
            EditorTool::Text,
            EditorTool::ColorPicker,
        ]
    }
}

#[derive(Clone, Debug)]
pub struct ToolState {
    pub active_tool: EditorTool,

    pub color: RGBA,

    pub line_width: f64,

    pub font_size: f64,

    pub is_drawing: bool,

    pub drag_start: Option<(f64, f64)>,

    pub drag_current: Option<(f64, f64)>,

    pub pointer_drag_offset: Option<(f64, f64)>,

    pub is_dragging_annotation: bool,

    pub moved_annotation: bool,
}

impl Default for ToolState {
    fn default() -> Self {
        Self {
            active_tool: EditorTool::Pointer,
            color: RGBA::new(1.0, 0.0, 0.0, 1.0),
            line_width: 3.0,
            font_size: 24.0,
            is_drawing: false,
            drag_start: None,
            drag_current: None,
            pointer_drag_offset: None,
            is_dragging_annotation: false,
            moved_annotation: false,
        }
    }
}

impl ToolState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_tool(&mut self, tool: EditorTool) {
        self.active_tool = tool;
        self.reset_drag();
    }

    pub fn set_color(&mut self, color: RGBA) {
        self.color = color;
    }

    pub fn set_line_width(&mut self, width: f64) {
        self.line_width = width.max(1.0).min(50.0);
    }

    pub fn set_font_size(&mut self, size: f64) {
        self.font_size = size.max(8.0).min(200.0);
    }

    pub fn start_drag(&mut self, x: f64, y: f64) {
        self.is_drawing = true;
        self.drag_start = Some((x, y));
        self.drag_current = Some((x, y));
    }

    pub fn update_drag(&mut self, x: f64, y: f64) {
        if self.is_drawing {
            self.drag_current = Some((x, y));
        }
    }

    pub fn end_drag(&mut self) -> Option<((f64, f64), (f64, f64))> {
        let result = if let (Some(start), Some(end)) = (self.drag_start, self.drag_current) {
            Some((start, end))
        } else {
            None
        };
        self.reset_drag();
        result
    }

    pub fn reset_drag(&mut self) {
        self.is_drawing = false;
        self.drag_start = None;
        self.drag_current = None;
        self.pointer_drag_offset = None;
        self.is_dragging_annotation = false;
        self.moved_annotation = false;
    }

    pub fn start_annotation_drag(
        &mut self,
        click_x: f64,
        click_y: f64,
        annotation_x: f64,
        annotation_y: f64,
    ) {
        self.is_dragging_annotation = true;
        self.drag_start = Some((click_x, click_y));
        self.drag_current = Some((click_x, click_y));

        self.pointer_drag_offset = Some((click_x - annotation_x, click_y - annotation_y));
        self.moved_annotation = false;
    }

    pub fn update_annotation_drag(&mut self, x: f64, y: f64) {
        if self.is_dragging_annotation {
            if let Some((last_x, last_y)) = self.drag_current {
                if (x - last_x).abs() > 0.1 || (y - last_y).abs() > 0.1 {
                    self.moved_annotation = true;
                }
            }
            self.drag_current = Some((x, y));
        }
    }

    pub fn end_annotation_drag(&mut self) -> Option<(f64, f64)> {
        if self.is_dragging_annotation {
            if let (Some((current_x, current_y)), Some((offset_x, offset_y))) =
                (self.drag_current, self.pointer_drag_offset)
            {
                let new_x = current_x - offset_x;
                let new_y = current_y - offset_y;
                self.reset_drag();
                return Some((new_x, new_y));
            }
        }
        self.reset_drag();
        None
    }

    pub fn get_drag_rect(&self) -> Option<(f64, f64, f64, f64)> {
        if let (Some((x1, y1)), Some((x2, y2))) = (self.drag_start, self.drag_current) {
            let x = x1.min(x2);
            let y = y1.min(y2);
            let w = (x2 - x1).abs();
            let h = (y2 - y1).abs();
            Some((x, y, w, h))
        } else {
            None
        }
    }
}
