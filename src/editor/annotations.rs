use gtk4::gdk::RGBA;

/// A point in 2D space
#[derive(Clone, Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// Rectangle annotation
#[derive(Clone, Debug)]
pub struct RectangleAnnotation {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: RGBA,
    pub line_width: f64,
    pub filled: bool,
}

impl RectangleAnnotation {
    pub fn new(x: f64, y: f64, width: f64, height: f64, color: RGBA, line_width: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
            color,
            line_width,
            filled: false,
        }
    }

    /// Create from two corner points, normalizing to positive width/height
    pub fn from_corners(x1: f64, y1: f64, x2: f64, y2: f64, color: RGBA, line_width: f64) -> Self {
        let x = x1.min(x2);
        let y = y1.min(y2);
        let width = (x2 - x1).abs();
        let height = (y2 - y1).abs();
        Self::new(x, y, width, height, color, line_width)
    }
}

/// Free drawing path annotation
#[derive(Clone, Debug)]
pub struct FreeDrawAnnotation {
    pub points: Vec<Point>,
    pub color: RGBA,
    pub line_width: f64,
}

impl FreeDrawAnnotation {
    pub fn new(color: RGBA, line_width: f64) -> Self {
        Self {
            points: Vec::new(),
            color,
            line_width,
        }
    }

    pub fn add_point(&mut self, x: f64, y: f64) {
        self.points.push(Point::new(x, y));
    }
}

/// Text annotation
#[derive(Clone, Debug)]
pub struct TextAnnotation {
    pub x: f64,
    pub y: f64,
    pub text: String,
    pub color: RGBA,
    pub font_size: f64,
    pub font_family: String,
}

impl TextAnnotation {
    pub fn new(x: f64, y: f64, text: String, color: RGBA, font_size: f64) -> Self {
        Self {
            x,
            y,
            text,
            color,
            font_size,
            font_family: "Sans".to_string(),
        }
    }
}

/// Enum representing all annotation types
#[derive(Clone, Debug)]
pub enum Annotation {
    Rectangle(RectangleAnnotation),
    FreeDraw(FreeDrawAnnotation),
    Text(TextAnnotation),
}

impl Annotation {
    /// Draw the annotation on a cairo context
    pub fn draw(&self, cr: &gtk4::cairo::Context, scale: f64, offset_x: f64, offset_y: f64) {
        match self {
            Annotation::Rectangle(rect) => {
                cr.set_source_rgba(
                    rect.color.red() as f64,
                    rect.color.green() as f64,
                    rect.color.blue() as f64,
                    rect.color.alpha() as f64,
                );
                cr.set_line_width(rect.line_width);

                let x = offset_x + rect.x * scale;
                let y = offset_y + rect.y * scale;
                let w = rect.width * scale;
                let h = rect.height * scale;

                cr.rectangle(x, y, w, h);

                if rect.filled {
                    let _ = cr.fill();
                } else {
                    let _ = cr.stroke();
                }
            }
            Annotation::FreeDraw(draw) => {
                if draw.points.len() < 2 {
                    return;
                }

                cr.set_source_rgba(
                    draw.color.red() as f64,
                    draw.color.green() as f64,
                    draw.color.blue() as f64,
                    draw.color.alpha() as f64,
                );
                cr.set_line_width(draw.line_width);
                cr.set_line_cap(gtk4::cairo::LineCap::Round);
                cr.set_line_join(gtk4::cairo::LineJoin::Round);

                let first = &draw.points[0];
                cr.move_to(offset_x + first.x * scale, offset_y + first.y * scale);

                for point in draw.points.iter().skip(1) {
                    cr.line_to(offset_x + point.x * scale, offset_y + point.y * scale);
                }

                let _ = cr.stroke();
            }
            Annotation::Text(text) => {
                cr.set_source_rgba(
                    text.color.red() as f64,
                    text.color.green() as f64,
                    text.color.blue() as f64,
                    text.color.alpha() as f64,
                );

                let font_size = text.font_size * scale;
                cr.set_font_size(font_size);

                let x = offset_x + text.x * scale;
                let y = offset_y + text.y * scale;

                cr.move_to(x, y);
                let _ = cr.show_text(&text.text);
            }
        }
    }
}

/// Collection of annotations with undo support
#[derive(Clone, Debug, Default)]
pub struct AnnotationList {
    annotations: Vec<Annotation>,
    current_annotation: Option<Annotation>,
}

impl AnnotationList {
    pub fn new() -> Self {
        Self {
            annotations: Vec::new(),
            current_annotation: None,
        }
    }

    pub fn add(&mut self, annotation: Annotation) {
        self.annotations.push(annotation);
    }

    pub fn set_current(&mut self, annotation: Option<Annotation>) {
        self.current_annotation = annotation;
    }

    pub fn commit_current(&mut self) {
        if let Some(annotation) = self.current_annotation.take() {
            self.annotations.push(annotation);
        }
    }

    pub fn clear_current(&mut self) {
        self.current_annotation = None;
    }

    pub fn undo(&mut self) -> bool {
        self.annotations.pop().is_some()
    }

    pub fn clear(&mut self) {
        self.annotations.clear();
        self.current_annotation = None;
    }

    pub fn iter(&self) -> impl Iterator<Item = &Annotation> {
        self.annotations.iter()
    }

    pub fn current(&self) -> Option<&Annotation> {
        self.current_annotation.as_ref()
    }

    pub fn draw_all(&self, cr: &gtk4::cairo::Context, scale: f64, offset_x: f64, offset_y: f64) {
        for annotation in &self.annotations {
            annotation.draw(cr, scale, offset_x, offset_y);
        }

        if let Some(current) = &self.current_annotation {
            current.draw(cr, scale, offset_x, offset_y);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.annotations.is_empty() && self.current_annotation.is_none()
    }

    pub fn len(&self) -> usize {
        self.annotations.len()
    }
}
