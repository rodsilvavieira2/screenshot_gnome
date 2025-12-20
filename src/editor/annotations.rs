use gtk4::gdk::RGBA;


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

    
    pub fn from_corners(x1: f64, y1: f64, x2: f64, y2: f64, color: RGBA, line_width: f64) -> Self {
        let x = x1.min(x2);
        let y = y1.min(y2);
        let width = (x2 - x1).abs();
        let height = (y2 - y1).abs();
        Self::new(x, y, width, height, color, line_width)
    }

    
    pub fn hit_test(&self, px: f64, py: f64) -> bool {
        let margin = self.line_width.max(5.0);

        if self.filled {
            
            px >= self.x - margin
                && px <= self.x + self.width + margin
                && py >= self.y - margin
                && py <= self.y + self.height + margin
        } else {
            
            let near_left = (px - self.x).abs() <= margin
                && py >= self.y - margin
                && py <= self.y + self.height + margin;
            let near_right = (px - (self.x + self.width)).abs() <= margin
                && py >= self.y - margin
                && py <= self.y + self.height + margin;
            let near_top = (py - self.y).abs() <= margin
                && px >= self.x - margin
                && px <= self.x + self.width + margin;
            let near_bottom = (py - (self.y + self.height)).abs() <= margin
                && px >= self.x - margin
                && px <= self.x + self.width + margin;

            near_left || near_right || near_top || near_bottom
        }
    }

    
    pub fn move_by(&mut self, dx: f64, dy: f64) {
        self.x += dx;
        self.y += dy;
    }

    
    pub fn set_position(&mut self, x: f64, y: f64) {
        self.x = x;
        self.y = y;
    }
}


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

    
    pub fn hit_test(&self, px: f64, py: f64) -> bool {
        let margin = self.line_width.max(8.0);

        for point in &self.points {
            let dx = px - point.x;
            let dy = py - point.y;
            if dx * dx + dy * dy <= margin * margin {
                return true;
            }
        }

        
        for i in 0..self.points.len().saturating_sub(1) {
            let p1 = &self.points[i];
            let p2 = &self.points[i + 1];
            if point_to_segment_distance(px, py, p1.x, p1.y, p2.x, p2.y) <= margin {
                return true;
            }
        }

        false
    }

    
    pub fn move_by(&mut self, dx: f64, dy: f64) {
        for point in &mut self.points {
            point.x += dx;
            point.y += dy;
        }
    }

    
    pub fn bounding_box(&self) -> Option<(f64, f64, f64, f64)> {
        if self.points.is_empty() {
            return None;
        }

        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;

        for point in &self.points {
            min_x = min_x.min(point.x);
            min_y = min_y.min(point.y);
            max_x = max_x.max(point.x);
            max_y = max_y.max(point.y);
        }

        Some((min_x, min_y, max_x - min_x, max_y - min_y))
    }
}


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

    
    pub fn hit_test(&self, px: f64, py: f64) -> bool {
        
        let approx_char_width = self.font_size * 0.6;
        let text_width = self.text.len() as f64 * approx_char_width;
        let text_height = self.font_size;

        
        let margin = 5.0;
        px >= self.x - margin
            && px <= self.x + text_width + margin
            && py >= self.y - text_height - margin
            && py <= self.y + margin
    }

    
    pub fn move_by(&mut self, dx: f64, dy: f64) {
        self.x += dx;
        self.y += dy;
    }

    
    pub fn set_position(&mut self, x: f64, y: f64) {
        self.x = x;
        self.y = y;
    }
}


fn point_to_segment_distance(px: f64, py: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let length_sq = dx * dx + dy * dy;

    if length_sq == 0.0 {
        
        let dpx = px - x1;
        let dpy = py - y1;
        return (dpx * dpx + dpy * dpy).sqrt();
    }

    
    let t = ((px - x1) * dx + (py - y1) * dy) / length_sq;
    let t = t.clamp(0.0, 1.0);

    let proj_x = x1 + t * dx;
    let proj_y = y1 + t * dy;

    let dpx = px - proj_x;
    let dpy = py - proj_y;
    (dpx * dpx + dpy * dpy).sqrt()
}


#[derive(Clone, Debug)]
pub enum Annotation {
    Rectangle(RectangleAnnotation),
    FreeDraw(FreeDrawAnnotation),
    Text(TextAnnotation),
}

impl Annotation {
    
    pub fn hit_test(&self, px: f64, py: f64) -> bool {
        match self {
            Annotation::Rectangle(rect) => rect.hit_test(px, py),
            Annotation::FreeDraw(draw) => draw.hit_test(px, py),
            Annotation::Text(text) => text.hit_test(px, py),
        }
    }

    
    pub fn move_by(&mut self, dx: f64, dy: f64) {
        match self {
            Annotation::Rectangle(rect) => rect.move_by(dx, dy),
            Annotation::FreeDraw(draw) => draw.move_by(dx, dy),
            Annotation::Text(text) => text.move_by(dx, dy),
        }
    }

    
    pub fn position(&self) -> (f64, f64) {
        match self {
            Annotation::Rectangle(rect) => (rect.x, rect.y),
            Annotation::FreeDraw(draw) => {
                if let Some(first) = draw.points.first() {
                    (first.x, first.y)
                } else {
                    (0.0, 0.0)
                }
            }
            Annotation::Text(text) => (text.x, text.y),
        }
    }

    
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

    
    pub fn draw_selected(
        &self,
        cr: &gtk4::cairo::Context,
        scale: f64,
        offset_x: f64,
        offset_y: f64,
    ) {
        
        self.draw(cr, scale, offset_x, offset_y);

        
        let (x, y, w, h) = match self {
            Annotation::Rectangle(rect) => (rect.x, rect.y, rect.width, rect.height),
            Annotation::FreeDraw(draw) => {
                if let Some((bx, by, bw, bh)) = draw.bounding_box() {
                    (bx, by, bw, bh)
                } else {
                    return;
                }
            }
            Annotation::Text(text) => {
                let approx_char_width = text.font_size * 0.6;
                let text_width = text.text.len() as f64 * approx_char_width;
                (text.x, text.y - text.font_size, text_width, text.font_size)
            }
        };

        let margin = 4.0;
        let dx = offset_x + (x - margin) * scale;
        let dy = offset_y + (y - margin) * scale;
        let dw = (w + margin * 2.0) * scale;
        let dh = (h + margin * 2.0) * scale;

        
        cr.set_source_rgba(0.2, 0.6, 1.0, 0.8);
        cr.set_line_width(2.0);
        cr.set_dash(&[6.0, 4.0], 0.0);
        cr.rectangle(dx, dy, dw, dh);
        let _ = cr.stroke();

        
        cr.set_dash(&[], 0.0);

        
        let handle_size = 8.0;
        cr.set_source_rgba(0.2, 0.6, 1.0, 1.0);

        
        cr.rectangle(
            dx - handle_size / 2.0,
            dy - handle_size / 2.0,
            handle_size,
            handle_size,
        );
        
        cr.rectangle(
            dx + dw - handle_size / 2.0,
            dy - handle_size / 2.0,
            handle_size,
            handle_size,
        );
        
        cr.rectangle(
            dx - handle_size / 2.0,
            dy + dh - handle_size / 2.0,
            handle_size,
            handle_size,
        );
        
        cr.rectangle(
            dx + dw - handle_size / 2.0,
            dy + dh - handle_size / 2.0,
            handle_size,
            handle_size,
        );
        let _ = cr.fill();
    }
}


#[derive(Clone, Debug, Default)]
pub struct AnnotationList {
    annotations: Vec<Annotation>,
    current_annotation: Option<Annotation>,
    
    selected_index: Option<usize>,
}

impl AnnotationList {
    pub fn new() -> Self {
        Self {
            annotations: Vec::new(),
            current_annotation: None,
            selected_index: None,
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
        self.selected_index = None;
        self.annotations.pop().is_some()
    }

    pub fn clear(&mut self) {
        self.annotations.clear();
        self.current_annotation = None;
        self.selected_index = None;
    }

    pub fn iter(&self) -> impl Iterator<Item = &Annotation> {
        self.annotations.iter()
    }

    pub fn current(&self) -> Option<&Annotation> {
        self.current_annotation.as_ref()
    }

    
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Annotation> {
        self.annotations.get_mut(index)
    }

    
    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    
    pub fn set_selected(&mut self, index: Option<usize>) {
        self.selected_index = index;
    }

    
    pub fn deselect(&mut self) {
        self.selected_index = None;
    }

    
    pub fn hit_test(&self, px: f64, py: f64) -> Option<usize> {
        
        for (i, annotation) in self.annotations.iter().enumerate().rev() {
            if annotation.hit_test(px, py) {
                return Some(i);
            }
        }
        None
    }

    
    pub fn move_selected(&mut self, dx: f64, dy: f64) -> bool {
        if let Some(index) = self.selected_index {
            if let Some(annotation) = self.annotations.get_mut(index) {
                annotation.move_by(dx, dy);
                return true;
            }
        }
        false
    }

    
    pub fn selected_position(&self) -> Option<(f64, f64)> {
        if let Some(index) = self.selected_index {
            self.annotations.get(index).map(|a| a.position())
        } else {
            None
        }
    }

    pub fn draw_all(&self, cr: &gtk4::cairo::Context, scale: f64, offset_x: f64, offset_y: f64) {
        for (i, annotation) in self.annotations.iter().enumerate() {
            if Some(i) == self.selected_index {
                annotation.draw_selected(cr, scale, offset_x, offset_y);
            } else {
                annotation.draw(cr, scale, offset_x, offset_y);
            }
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
