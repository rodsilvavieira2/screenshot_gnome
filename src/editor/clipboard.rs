use gtk4::gdk::Texture;
use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::prelude::*;


pub type ClipboardResult<T> = Result<T, ClipboardError>;


#[derive(Debug)]
pub enum ClipboardError {
    NoDisplay,
    NoClipboard,
    NoImage,
    TextureCreationFailed,
}

impl std::fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClipboardError::NoDisplay => write!(f, "No display available"),
            ClipboardError::NoClipboard => write!(f, "Could not access clipboard"),
            ClipboardError::NoImage => write!(f, "No image to copy"),
            ClipboardError::TextureCreationFailed => {
                write!(f, "Failed to create texture from image")
            }
        }
    }
}

impl std::error::Error for ClipboardError {}


pub fn copy_pixbuf_to_clipboard(
    pixbuf: &Pixbuf,
    display: &gtk4::gdk::Display,
) -> ClipboardResult<()> {
    let clipboard = display.clipboard();

    
    let texture = Texture::for_pixbuf(pixbuf);

    
    clipboard.set_texture(&texture);

    Ok(())
}


pub fn copy_to_clipboard_from_widget(
    pixbuf: &Pixbuf,
    widget: &impl IsA<gtk4::Widget>,
) -> ClipboardResult<()> {
    let display = widget.display();
    copy_pixbuf_to_clipboard(pixbuf, &display)
}


pub fn copy_text_to_clipboard(text: &str, display: &gtk4::gdk::Display) -> ClipboardResult<()> {
    let clipboard = display.clipboard();
    clipboard.set_text(text);
    Ok(())
}


pub struct ClipboardManager {
    display: gtk4::gdk::Display,
}

impl ClipboardManager {
    
    pub fn new(display: gtk4::gdk::Display) -> Self {
        Self { display }
    }

    
    pub fn from_widget(widget: &impl IsA<gtk4::Widget>) -> Self {
        Self {
            display: widget.display(),
        }
    }

    
    pub fn copy_image(&self, pixbuf: &Pixbuf) -> ClipboardResult<()> {
        copy_pixbuf_to_clipboard(pixbuf, &self.display)
    }

    
    pub fn copy_text(&self, text: &str) -> ClipboardResult<()> {
        copy_text_to_clipboard(text, &self.display)
    }

    
    pub fn copy_color(&self, color: &gtk4::gdk::RGBA) -> ClipboardResult<()> {
        let hex = format!(
            "#{:02X}{:02X}{:02X}",
            (color.red() * 255.0) as u8,
            (color.green() * 255.0) as u8,
            (color.blue() * 255.0) as u8
        );
        self.copy_text(&hex)
    }
}
