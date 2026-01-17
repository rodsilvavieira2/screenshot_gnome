use gtk4::gdk::Texture;
use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::prelude::*;

pub type ClipboardResult<T> = Result<T, ClipboardError>;

#[derive(Debug)]
pub enum ClipboardError {}

impl std::fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Clipboard error")
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

pub struct ClipboardManager {
    display: gtk4::gdk::Display,
}

impl ClipboardManager {
    pub fn from_widget(widget: &impl IsA<gtk4::Widget>) -> Self {
        Self {
            display: widget.display(),
        }
    }

    pub fn copy_image(&self, pixbuf: &Pixbuf) -> ClipboardResult<()> {
        copy_pixbuf_to_clipboard(pixbuf, &self.display)
    }
}
