use gtk4::gdk::RGBA;
use gtk4::gdk_pixbuf::Pixbuf;


#[derive(Clone, Debug)]
pub struct PickedColor {
    
    pub color: RGBA,
    
    pub x: i32,
    pub y: i32,
}

impl PickedColor {
    
    pub fn to_hex(&self) -> String {
        format!(
            "#{:02X}{:02X}{:02X}",
            (self.color.red() * 255.0) as u8,
            (self.color.green() * 255.0) as u8,
            (self.color.blue() * 255.0) as u8
        )
    }

    
    pub fn to_rgb(&self) -> (u8, u8, u8) {
        (
            (self.color.red() * 255.0) as u8,
            (self.color.green() * 255.0) as u8,
            (self.color.blue() * 255.0) as u8,
        )
    }

    
    pub fn to_rgba(&self) -> (u8, u8, u8, u8) {
        (
            (self.color.red() * 255.0) as u8,
            (self.color.green() * 255.0) as u8,
            (self.color.blue() * 255.0) as u8,
            (self.color.alpha() * 255.0) as u8,
        )
    }
}


#[derive(Debug)]
pub enum ColorPickError {
    OutOfBounds,
    InvalidPixbuf,
}

impl std::fmt::Display for ColorPickError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorPickError::OutOfBounds => write!(f, "Coordinates are outside the image bounds"),
            ColorPickError::InvalidPixbuf => write!(f, "Invalid pixbuf data"),
        }
    }
}

impl std::error::Error for ColorPickError {}


pub fn pick_color_from_pixbuf(
    pixbuf: &Pixbuf,
    x: i32,
    y: i32,
) -> Result<PickedColor, ColorPickError> {
    let width = pixbuf.width();
    let height = pixbuf.height();

    
    if x < 0 || x >= width || y < 0 || y >= height {
        return Err(ColorPickError::OutOfBounds);
    }

    let n_channels = pixbuf.n_channels() as usize;
    let rowstride = pixbuf.rowstride() as usize;
    let has_alpha = pixbuf.has_alpha();

    
    let pixels = unsafe { pixbuf.pixels() };

    
    let offset = (y as usize) * rowstride + (x as usize) * n_channels;

    if offset + n_channels > pixels.len() {
        return Err(ColorPickError::InvalidPixbuf);
    }

    let r = pixels[offset] as f32 / 255.0;
    let g = pixels[offset + 1] as f32 / 255.0;
    let b = pixels[offset + 2] as f32 / 255.0;
    let a = if has_alpha && n_channels >= 4 {
        pixels[offset + 3] as f32 / 255.0
    } else {
        1.0
    };

    Ok(PickedColor {
        color: RGBA::new(r, g, b, a),
        x,
        y,
    })
}


pub fn pick_color_at_display_coords(
    pixbuf: &Pixbuf,
    display_x: f64,
    display_y: f64,
    scale: f64,
    offset_x: f64,
    offset_y: f64,
) -> Result<PickedColor, ColorPickError> {
    
    let img_x = ((display_x - offset_x) / scale) as i32;
    let img_y = ((display_y - offset_y) / scale) as i32;

    pick_color_from_pixbuf(pixbuf, img_x, img_y)
}


pub fn pick_average_color(
    pixbuf: &Pixbuf,
    center_x: i32,
    center_y: i32,
    radius: i32,
) -> Result<PickedColor, ColorPickError> {
    let width = pixbuf.width();
    let height = pixbuf.height();

    
    if center_x < 0 || center_x >= width || center_y < 0 || center_y >= height {
        return Err(ColorPickError::OutOfBounds);
    }

    let n_channels = pixbuf.n_channels() as usize;
    let rowstride = pixbuf.rowstride() as usize;
    let has_alpha = pixbuf.has_alpha();
    let pixels = unsafe { pixbuf.pixels() };

    let mut total_r: f64 = 0.0;
    let mut total_g: f64 = 0.0;
    let mut total_b: f64 = 0.0;
    let mut total_a: f64 = 0.0;
    let mut count: usize = 0;

    let start_x = (center_x - radius).max(0);
    let end_x = (center_x + radius).min(width - 1);
    let start_y = (center_y - radius).max(0);
    let end_y = (center_y + radius).min(height - 1);

    for py in start_y..=end_y {
        for px in start_x..=end_x {
            let offset = (py as usize) * rowstride + (px as usize) * n_channels;

            if offset + n_channels <= pixels.len() {
                total_r += pixels[offset] as f64;
                total_g += pixels[offset + 1] as f64;
                total_b += pixels[offset + 2] as f64;
                total_a += if has_alpha && n_channels >= 4 {
                    pixels[offset + 3] as f64
                } else {
                    255.0
                };
                count += 1;
            }
        }
    }

    if count == 0 {
        return Err(ColorPickError::InvalidPixbuf);
    }

    let avg_r = (total_r / count as f64 / 255.0) as f32;
    let avg_g = (total_g / count as f64 / 255.0) as f32;
    let avg_b = (total_b / count as f64 / 255.0) as f32;
    let avg_a = (total_a / count as f64 / 255.0) as f32;

    Ok(PickedColor {
        color: RGBA::new(avg_r, avg_g, avg_b, avg_a),
        x: center_x,
        y: center_y,
    })
}


#[derive(Clone, Debug, Default)]
pub struct ColorPickerState {
    
    pub picked_color: Option<PickedColor>,
    
    pub is_active: bool,
}

impl ColorPickerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn activate(&mut self) {
        self.is_active = true;
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    pub fn set_picked_color(&mut self, color: PickedColor) {
        self.picked_color = Some(color);
    }

    pub fn clear(&mut self) {
        self.picked_color = None;
    }

    pub fn get_color(&self) -> Option<RGBA> {
        self.picked_color.as_ref().map(|p| p.color)
    }
}
