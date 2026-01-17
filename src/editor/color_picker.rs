use gtk4::gdk::RGBA;
use gtk4::gdk_pixbuf::Pixbuf;

#[derive(Clone, Debug)]
pub struct PickedColor {
    pub color: RGBA,
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
    })
}

#[derive(Clone, Debug, Default)]
pub struct ColorPickerState {
    pub picked_color: Option<PickedColor>,
}

impl ColorPickerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_picked_color(&mut self, color: PickedColor) {
        self.picked_color = Some(color);
    }

    pub fn clear(&mut self) {
        self.picked_color = None;
    }
}
