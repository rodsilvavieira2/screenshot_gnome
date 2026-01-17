use gtk4::gdk_pixbuf::Pixbuf;
use log::{debug, info};

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

    /// Returns a debug string with detailed window information
    pub fn debug_info(&self) -> String {
        format!(
            "Window[id={}, pid={}, z={}, focused={}, maximized={}, minimized={}]: {}",
            self.id,
            self.pid,
            self.z,
            self.is_focused,
            self.is_maximized,
            self.is_minimized,
            self.display_label()
        )
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

pub fn list_capturable_windows() -> Result<Vec<WindowInfo>, WindowCaptureError> {
    let session = DesktopSession::detect();
    info!(
        "Detected session: {} (using {} backend)",
        session,
        session.window_list_backend()
    );

    let windows = window_backends::list_windows_for_session(&session)?;
    debug!("Found {} windows in total", windows.len());

    let capturable: Vec<WindowInfo> = windows.into_iter().filter(|w| !w.is_minimized).collect();
    debug!(
        "{} windows are capturable (not minimized)",
        capturable.len()
    );

    Ok(capturable)
}

pub fn capture_window(window_info: &WindowInfo) -> Result<WindowCaptureResult, WindowCaptureError> {
    let session = DesktopSession::detect();
    info!(
        "Capturing window '{}' using {} backend",
        window_info.display_label(),
        session.window_list_backend()
    );
    debug!("Window details: {:?}", window_info);

    window_backends::capture_window_for_session(&session, window_info)
}
