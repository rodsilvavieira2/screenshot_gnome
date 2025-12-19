use gtk4::gdk::Texture;
use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::prelude::*;

/// Result type for clipboard operations
pub type ClipboardResult<T> = Result<T, ClipboardError>;

/// Errors that can occur during clipboard operations
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

/// Copy a pixbuf to the system clipboard
pub fn copy_pixbuf_to_clipboard(
    pixbuf: &Pixbuf,
    display: &gtk4::gdk::Display,
) -> ClipboardResult<()> {
    let clipboard = display.clipboard();

    // Create a texture from the pixbuf
    let texture = Texture::for_pixbuf(pixbuf);

    // Set the texture on the clipboard
    clipboard.set_texture(&texture);

    Ok(())
}

/// Copy a pixbuf to clipboard using a widget's display
pub fn copy_to_clipboard_from_widget(
    pixbuf: &Pixbuf,
    widget: &impl IsA<gtk4::Widget>,
) -> ClipboardResult<()> {
    let display = widget.display();
    copy_pixbuf_to_clipboard(pixbuf, &display)
}

/// Copy text to the system clipboard
pub fn copy_text_to_clipboard(text: &str, display: &gtk4::gdk::Display) -> ClipboardResult<()> {
    let clipboard = display.clipboard();
    clipboard.set_text(text);
    Ok(())
}

/// Helper struct for clipboard operations
pub struct ClipboardManager {
    display: gtk4::gdk::Display,
}

impl ClipboardManager {
    /// Create a new clipboard manager from a display
    pub fn new(display: gtk4::gdk::Display) -> Self {
        Self { display }
    }

    /// Create from a widget's display
    pub fn from_widget(widget: &impl IsA<gtk4::Widget>) -> Self {
        Self {
            display: widget.display(),
        }
    }

    /// Copy a pixbuf image to the clipboard
    pub fn copy_image(&self, pixbuf: &Pixbuf) -> ClipboardResult<()> {
        copy_pixbuf_to_clipboard(pixbuf, &self.display)
    }

    /// Copy text to the clipboard
    pub fn copy_text(&self, text: &str) -> ClipboardResult<()> {
        copy_text_to_clipboard(text, &self.display)
    }

    /// Copy a color as hex string to the clipboard
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
