//! Window capture module using xcap library
//!
//! This module provides functionality to list available windows and capture their contents.
//! Compatible with xcap version 0.0.14.

#![allow(dead_code)]

use gtk4::gdk_pixbuf::{Colorspace, Pixbuf};
use gtk4::glib;
use xcap::Window;

/// Information about a capturable window
#[derive(Debug, Clone)]
pub struct WindowInfo {
    /// Window ID
    pub id: u32,
    /// Application name
    pub app_name: String,
    /// Window title
    pub title: String,
    /// Window X position
    pub x: i32,
    /// Window Y position
    pub y: i32,
    /// Window width
    pub width: u32,
    /// Window height
    pub height: u32,
    /// Whether the window is minimized
    pub is_minimized: bool,
    /// Whether the window is maximized
    pub is_maximized: bool,
}

impl WindowInfo {
    /// Create WindowInfo from an xcap Window
    fn from_xcap_window(window: &Window) -> Self {
        Self {
            id: window.id(),
            app_name: window.app_name().to_string(),
            title: window.title().to_string(),
            x: window.x(),
            y: window.y(),
            width: window.width(),
            height: window.height(),
            is_minimized: window.is_minimized(),
            is_maximized: window.is_maximized(),
        }
    }

    /// Get a display label for this window
    pub fn display_label(&self) -> String {
        if self.title.is_empty() {
            format!("{} (ID: {})", self.app_name, self.id)
        } else {
            format!("{} — {}", self.title, self.app_name)
        }
    }

    /// Get an icon name hint based on app name
    pub fn icon_name_hint(&self) -> &str {
        if self.app_name.is_empty() {
            "application-x-executable-symbolic"
        } else {
            // Return app_name as-is; caller should lowercase for icon lookup
            &self.app_name
        }
    }
}

/// Result of a window capture operation
pub struct WindowCaptureResult {
    /// The captured image as a GdkPixbuf
    pub pixbuf: Pixbuf,
    /// Information about the captured window
    pub window_info: WindowInfo,
}

/// Error type for window capture operations
#[derive(Debug)]
pub enum WindowCaptureError {
    /// Failed to enumerate windows
    EnumerationFailed(String),
    /// Window not found
    WindowNotFound,
    /// Failed to capture window
    CaptureFailed(String),
    /// Failed to convert image to pixbuf
    ConversionFailed(String),
    /// Window is minimized and cannot be captured
    WindowMinimized,
}

impl std::fmt::Display for WindowCaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EnumerationFailed(msg) => write!(f, "Failed to enumerate windows: {}", msg),
            Self::WindowNotFound => write!(f, "Window not found"),
            Self::CaptureFailed(msg) => write!(f, "Failed to capture window: {}", msg),
            Self::ConversionFailed(msg) => write!(f, "Failed to convert image: {}", msg),
            Self::WindowMinimized => write!(f, "Cannot capture minimized window"),
        }
    }
}

impl std::error::Error for WindowCaptureError {}

/// Get a list of all available windows that can be captured
///
/// Returns a list of WindowInfo for all non-minimized windows.
pub fn list_capturable_windows() -> Result<Vec<WindowInfo>, WindowCaptureError> {
    let windows =
        Window::all().map_err(|e| WindowCaptureError::EnumerationFailed(e.to_string()))?;

    let window_infos: Vec<WindowInfo> = windows
        .iter()
        .filter(|w| !w.is_minimized())
        .map(WindowInfo::from_xcap_window)
        .collect();

    Ok(window_infos)
}

/// Get a list of all windows including minimized ones
///
/// Useful for displaying a complete window list to the user.
pub fn list_all_windows() -> Result<Vec<WindowInfo>, WindowCaptureError> {
    let windows =
        Window::all().map_err(|e| WindowCaptureError::EnumerationFailed(e.to_string()))?;

    let window_infos: Vec<WindowInfo> = windows.iter().map(WindowInfo::from_xcap_window).collect();

    Ok(window_infos)
}

/// Capture a specific window by its index in the capturable windows list
///
/// # Arguments
/// * `index` - Index into the list returned by `list_capturable_windows()`
///
/// # Returns
/// * `Ok(WindowCaptureResult)` - The captured window image and info
/// * `Err(WindowCaptureError)` - If capture fails
pub fn capture_window_by_index(index: usize) -> Result<WindowCaptureResult, WindowCaptureError> {
    let windows =
        Window::all().map_err(|e| WindowCaptureError::EnumerationFailed(e.to_string()))?;

    // Filter to non-minimized windows
    let capturable_windows: Vec<_> = windows.into_iter().filter(|w| !w.is_minimized()).collect();

    let window = capturable_windows
        .get(index)
        .ok_or(WindowCaptureError::WindowNotFound)?;

    capture_window_internal(window)
}

/// Capture a specific window by its ID
///
/// # Arguments
/// * `window_id` - The window ID to capture
///
/// # Returns
/// * `Ok(WindowCaptureResult)` - The captured window image and info
/// * `Err(WindowCaptureError)` - If capture fails
pub fn capture_window_by_id(window_id: u32) -> Result<WindowCaptureResult, WindowCaptureError> {
    let windows =
        Window::all().map_err(|e| WindowCaptureError::EnumerationFailed(e.to_string()))?;

    let window = windows
        .into_iter()
        .find(|w| w.id() == window_id)
        .ok_or(WindowCaptureError::WindowNotFound)?;

    if window.is_minimized() {
        return Err(WindowCaptureError::WindowMinimized);
    }

    capture_window_internal(&window)
}

/// Internal function to capture a window
fn capture_window_internal(window: &Window) -> Result<WindowCaptureResult, WindowCaptureError> {
    let window_info = WindowInfo::from_xcap_window(window);

    let image = window
        .capture_image()
        .map_err(|e| WindowCaptureError::CaptureFailed(e.to_string()))?;

    let pixbuf = rgba_image_to_pixbuf(image)?;

    Ok(WindowCaptureResult {
        pixbuf,
        window_info,
    })
}

/// Convert an RGBA image to a GdkPixbuf
fn rgba_image_to_pixbuf(image: image::RgbaImage) -> Result<Pixbuf, WindowCaptureError> {
    let width = image.width() as i32;
    let height = image.height() as i32;
    let stride = width * 4; // 4 bytes per pixel (RGBA)
    let pixels = image.into_raw();
    let bytes = glib::Bytes::from(&pixels);

    Ok(Pixbuf::from_bytes(
        &bytes,
        Colorspace::Rgb,
        true,
        8,
        width,
        height,
        stride,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_info_display_label() {
        let info = WindowInfo {
            id: 1,
            app_name: "firefox".to_string(),
            title: "Mozilla Firefox".to_string(),
            x: 0,
            y: 0,
            width: 800,
            height: 600,
            is_minimized: false,
            is_maximized: false,
        };

        assert_eq!(info.display_label(), "Mozilla Firefox — firefox");
    }

    #[test]
    fn test_window_info_display_label_no_title() {
        let info = WindowInfo {
            id: 1,
            app_name: "firefox".to_string(),
            title: "".to_string(),
            x: 0,
            y: 0,
            width: 800,
            height: 600,
            is_minimized: false,
            is_maximized: false,
        };

        assert_eq!(info.display_label(), "firefox (ID: 1)");
    }

    #[test]
    fn test_icon_name_hint() {
        let info = WindowInfo {
            id: 1,
            app_name: "Firefox".to_string(),
            title: "".to_string(),
            x: 0,
            y: 0,
            width: 800,
            height: 600,
            is_minimized: false,
            is_maximized: false,
        };

        assert_eq!(info.icon_name_hint(), "Firefox");
    }

    #[test]
    fn test_icon_name_hint_empty() {
        let info = WindowInfo {
            id: 1,
            app_name: "".to_string(),
            title: "".to_string(),
            x: 0,
            y: 0,
            width: 800,
            height: 600,
            is_minimized: false,
            is_maximized: false,
        };

        assert_eq!(info.icon_name_hint(), "application-x-executable-symbolic");
    }
}
