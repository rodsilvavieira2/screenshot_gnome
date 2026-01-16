#![allow(dead_code)]

use gtk4::gdk_pixbuf::{Colorspace, Pixbuf};
use gtk4::glib;
use xcap::Window;

use super::desktop::DesktopSession;
use super::window_backends;

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub id: u32,

    pub pid: u32,

    pub app_name: String,

    pub title: String,

    pub x: i32,

    pub y: i32,

    pub z: i32,

    pub width: u32,

    pub height: u32,

    pub is_minimized: bool,

    pub is_maximized: bool,

    pub is_focused: bool,
}

impl WindowInfo {
    fn from_xcap_window(window: &Window) -> Result<Self, String> {
        Ok(Self {
            id: window.id().map_err(|e| e.to_string())?,
            pid: window.pid().map_err(|e| e.to_string())?,
            app_name: window.app_name().map_err(|e| e.to_string())?,
            title: window.title().map_err(|e| e.to_string())?,
            x: window.x().map_err(|e| e.to_string())?,
            y: window.y().map_err(|e| e.to_string())?,
            z: window.z().map_err(|e| e.to_string())?,
            width: window.width().map_err(|e| e.to_string())?,
            height: window.height().map_err(|e| e.to_string())?,
            is_minimized: window.is_minimized().map_err(|e| e.to_string())?,
            is_maximized: window.is_maximized().map_err(|e| e.to_string())?,
            is_focused: window.is_focused().map_err(|e| e.to_string())?,
        })
    }

    pub fn display_label(&self) -> String {
        if self.title.is_empty() {
            format!("{} (ID: {})", self.app_name, self.id)
        } else {
            format!("{} — {}", self.title, self.app_name)
        }
    }

    pub fn icon_name_hint(&self) -> &str {
        if self.app_name.is_empty() {
            "application-x-executable-symbolic"
        } else {
            &self.app_name
        }
    }
}

pub struct WindowCaptureResult {
    pub pixbuf: Pixbuf,

    pub window_info: WindowInfo,
}

#[derive(Debug)]
pub enum WindowCaptureError {
    EnumerationFailed(String),

    WindowNotFound,

    CaptureFailed(String),

    ConversionFailed(String),

    WindowMinimized,

    InfoFailed(String),
}

impl std::fmt::Display for WindowCaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EnumerationFailed(msg) => write!(f, "Failed to enumerate windows: {}", msg),
            Self::WindowNotFound => write!(f, "Window not found"),
            Self::CaptureFailed(msg) => write!(f, "Failed to capture window: {}", msg),
            Self::ConversionFailed(msg) => write!(f, "Failed to convert image: {}", msg),
            Self::WindowMinimized => write!(f, "Cannot capture minimized window"),
            Self::InfoFailed(msg) => write!(f, "Failed to get window info: {}", msg),
        }
    }
}

impl std::error::Error for WindowCaptureError {}

pub fn list_capturable_windows() -> Result<Vec<WindowInfo>, WindowCaptureError> {
    let session = DesktopSession::detect();
    println!(
        "Detected session: {} (using {} backend)",
        session,
        session.window_list_backend()
    );

    let windows = window_backends::list_windows_for_session(&session)?;

    let capturable: Vec<WindowInfo> = windows.into_iter().filter(|w| !w.is_minimized).collect();

    Ok(capturable)
}

pub fn list_capturable_windows_xcap() -> Result<Vec<WindowInfo>, WindowCaptureError> {
    let windows =
        Window::all().map_err(|e| WindowCaptureError::EnumerationFailed(e.to_string()))?;

    let mut window_infos = Vec::new();

    for window in &windows {
        println!("Window: {}", window.title().unwrap_or_default());

        match WindowInfo::from_xcap_window(window) {
            Ok(info) => window_infos.push(info),
            Err(e) => eprintln!("Warning: Failed to get info for a window: {}", e),
        }
    }

    Ok(window_infos)
}

pub fn list_all_windows() -> Result<Vec<WindowInfo>, WindowCaptureError> {
    let session = DesktopSession::detect();
    window_backends::list_windows_for_session(&session)
}

pub fn list_all_windows_xcap() -> Result<Vec<WindowInfo>, WindowCaptureError> {
    let windows =
        Window::all().map_err(|e| WindowCaptureError::EnumerationFailed(e.to_string()))?;

    let mut window_infos = Vec::new();

    for window in &windows {
        match WindowInfo::from_xcap_window(window) {
            Ok(info) => window_infos.push(info),
            Err(e) => eprintln!("Warning: Failed to get info for a window: {}", e),
        }
    }

    Ok(window_infos)
}

/// Returns information about the current desktop session.
pub fn get_desktop_session() -> DesktopSession {
    DesktopSession::detect()
}

/// Captures a window using the smart backend selection based on WindowInfo.
///
/// This function uses the appropriate backend for the current desktop environment
/// to capture the window. It should be used with WindowInfo obtained from
/// `list_capturable_windows()`.
pub fn capture_window(window_info: &WindowInfo) -> Result<WindowCaptureResult, WindowCaptureError> {
    let session = DesktopSession::detect();
    println!(
        "Capturing window '{}' using {} backend",
        window_info.display_label(),
        session.window_list_backend()
    );

    window_backends::capture_window_for_session(&session, window_info)
}

pub fn capture_window_by_index(index: usize) -> Result<WindowCaptureResult, WindowCaptureError> {
    let windows =
        Window::all().map_err(|e| WindowCaptureError::EnumerationFailed(e.to_string()))?;

    let capturable_windows: Vec<_> = windows
        .into_iter()
        .filter(|w| !w.is_minimized().unwrap_or(true))
        .collect();

    let window = capturable_windows
        .get(index)
        .ok_or(WindowCaptureError::WindowNotFound)?;

    capture_window_internal(window)
}

pub fn capture_window_by_id(window_id: u32) -> Result<WindowCaptureResult, WindowCaptureError> {
    let windows =
        Window::all().map_err(|e| WindowCaptureError::EnumerationFailed(e.to_string()))?;

    let window = windows
        .into_iter()
        .find(|w| w.id().ok() == Some(window_id))
        .ok_or(WindowCaptureError::WindowNotFound)?;

    if window.is_minimized().unwrap_or(false) {
        return Err(WindowCaptureError::WindowMinimized);
    }

    capture_window_internal(&window)
}

fn capture_window_internal(window: &Window) -> Result<WindowCaptureResult, WindowCaptureError> {
    let window_info =
        WindowInfo::from_xcap_window(window).map_err(|e| WindowCaptureError::InfoFailed(e))?;

    let image = window
        .capture_image()
        .map_err(|e| WindowCaptureError::CaptureFailed(e.to_string()))?;

    let pixbuf = rgba_image_to_pixbuf(image)?;

    Ok(WindowCaptureResult {
        pixbuf,
        window_info,
    })
}

fn rgba_image_to_pixbuf(image: image::RgbaImage) -> Result<Pixbuf, WindowCaptureError> {
    let width = image.width() as i32;
    let height = image.height() as i32;
    let stride = width * 4;
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
            pid: 100,
            app_name: "firefox".to_string(),
            title: "Mozilla Firefox".to_string(),
            x: 0,
            y: 0,
            z: 0,
            width: 800,
            height: 600,
            is_minimized: false,
            is_maximized: false,
            is_focused: true,
        };

        assert_eq!(info.display_label(), "Mozilla Firefox — firefox");
    }

    #[test]
    fn test_window_info_display_label_no_title() {
        let info = WindowInfo {
            id: 1,
            pid: 100,
            app_name: "firefox".to_string(),
            title: "".to_string(),
            x: 0,
            y: 0,
            z: 0,
            width: 800,
            height: 600,
            is_minimized: false,
            is_maximized: false,
            is_focused: true,
        };

        assert_eq!(info.display_label(), "firefox (ID: 1)");
    }

    #[test]
    fn test_icon_name_hint() {
        let info = WindowInfo {
            id: 1,
            pid: 100,
            app_name: "Firefox".to_string(),
            title: "".to_string(),
            x: 0,
            y: 0,
            z: 0,
            width: 800,
            height: 600,
            is_minimized: false,
            is_maximized: false,
            is_focused: true,
        };

        assert_eq!(info.icon_name_hint(), "Firefox");
    }

    #[test]
    fn test_icon_name_hint_empty() {
        let info = WindowInfo {
            id: 1,
            pid: 100,
            app_name: "".to_string(),
            title: "".to_string(),
            x: 0,
            y: 0,
            z: 0,
            width: 800,
            height: 600,
            is_minimized: false,
            is_maximized: false,
            is_focused: true,
        };

        assert_eq!(info.icon_name_hint(), "application-x-executable-symbolic");
    }
}
