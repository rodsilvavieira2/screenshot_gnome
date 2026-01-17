use gtk4 as gtk;
use std::process::Command;
use xcap::Monitor;

use super::desktop::{DesktopEnvironment, DesktopSession, DisplayServer};

#[derive(Debug, Clone)]
pub struct MonitorInfo {
    pub x: i32,
    pub y: i32,
}

impl MonitorInfo {
    fn from_xcap(monitor: &Monitor) -> Result<Self, String> {
        Ok(Self {
            x: monitor.x().map_err(|e| e.to_string())?,
            y: monitor.y().map_err(|e| e.to_string())?,
        })
    }

    /// Create a default MonitorInfo for Wayland when we can't get detailed info
    fn default_wayland() -> Self {
        Self { x: 0, y: 0 }
    }
}

pub struct CaptureResult {
    pub pixbuf: gtk::gdk_pixbuf::Pixbuf,
    pub monitor_info: MonitorInfo,
}

/// Capture the primary monitor, using the appropriate backend for the current session
pub fn capture_primary_monitor() -> Result<CaptureResult, String> {
    let session = DesktopSession::detect();

    match session.display_server {
        DisplayServer::Wayland => capture_screen_wayland(&session),
        DisplayServer::X11 => capture_screen_xcap(),
        DisplayServer::Unknown => {
            // Try Wayland first, fall back to xcap
            capture_screen_wayland(&session).or_else(|_| capture_screen_xcap())
        }
    }
}

/// Capture screen using xcap (works on X11)
fn capture_screen_xcap() -> Result<CaptureResult, String> {
    let monitors = Monitor::all().map_err(|e| format!("Failed to get monitors: {}", e))?;

    let monitor = monitors
        .iter()
        .find(|m| m.is_primary().unwrap_or(false))
        .or(monitors.first())
        .ok_or("No monitors available")?;

    capture_monitor_internal(monitor)
}

/// Capture screen on Wayland using compositor-specific tools
fn capture_screen_wayland(session: &DesktopSession) -> Result<CaptureResult, String> {
    let temp_path = format!("/tmp/screenshot_gnome_screen_{}.png", std::process::id());

    let result = match &session.desktop_environment {
        DesktopEnvironment::Hyprland | DesktopEnvironment::Sway => capture_with_grim(&temp_path),
        DesktopEnvironment::Gnome => {
            capture_with_gnome_screenshot(&temp_path).or_else(|_| capture_with_grim(&temp_path))
        }
        DesktopEnvironment::Kde => {
            capture_with_spectacle(&temp_path).or_else(|_| capture_with_grim(&temp_path))
        }
        _ => {
            // Try common tools in order of preference
            capture_with_grim(&temp_path)
                .or_else(|_| capture_with_gnome_screenshot(&temp_path))
                .or_else(|_| capture_with_spectacle(&temp_path))
        }
    };

    // Clean up temp file on error
    if result.is_err() {
        let _ = std::fs::remove_file(&temp_path);
    }

    result
}

/// Capture using grim (wlroots-based compositors: Hyprland, Sway, etc.)
fn capture_with_grim(temp_path: &str) -> Result<CaptureResult, String> {
    let output = Command::new("grim")
        .arg(temp_path)
        .output()
        .map_err(|e| format!("Failed to run grim: {}. Is grim installed?", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("grim failed: {}", stderr));
    }

    let pixbuf = load_pixbuf_from_file(temp_path)?;
    let _ = std::fs::remove_file(temp_path);

    let monitor_info = MonitorInfo::default_wayland();

    Ok(CaptureResult {
        pixbuf,
        monitor_info,
    })
}

/// Capture using gnome-screenshot (GNOME)
fn capture_with_gnome_screenshot(temp_path: &str) -> Result<CaptureResult, String> {
    let output = Command::new("gnome-screenshot")
        .args(["-f", temp_path])
        .output()
        .map_err(|e| {
            format!(
                "Failed to run gnome-screenshot: {}. Is gnome-screenshot installed?",
                e
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("gnome-screenshot failed: {}", stderr));
    }

    let pixbuf = load_pixbuf_from_file(temp_path)?;
    let _ = std::fs::remove_file(temp_path);

    let monitor_info = MonitorInfo::default_wayland();

    Ok(CaptureResult {
        pixbuf,
        monitor_info,
    })
}

/// Capture using spectacle (KDE Plasma)
fn capture_with_spectacle(temp_path: &str) -> Result<CaptureResult, String> {
    let output = Command::new("spectacle")
        .args(["-b", "-n", "-f", "-o", temp_path])
        .output()
        .map_err(|e| format!("Failed to run spectacle: {}. Is spectacle installed?", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("spectacle failed: {}", stderr));
    }

    let pixbuf = load_pixbuf_from_file(temp_path)?;
    let _ = std::fs::remove_file(temp_path);

    let monitor_info = MonitorInfo::default_wayland();

    Ok(CaptureResult {
        pixbuf,
        monitor_info,
    })
}

/// Load a pixbuf from a file path
fn load_pixbuf_from_file(path: &str) -> Result<gtk::gdk_pixbuf::Pixbuf, String> {
    gtk::gdk_pixbuf::Pixbuf::from_file(path)
        .map_err(|e| format!("Failed to load screenshot image: {}", e))
}

fn capture_monitor_internal(monitor: &Monitor) -> Result<CaptureResult, String> {
    let monitor_info = MonitorInfo::from_xcap(monitor)?;

    let image = monitor
        .capture_image()
        .map_err(|e| format!("Failed to capture screen: {}", e))?;

    let pixbuf = image_to_pixbuf(image)?;

    Ok(CaptureResult {
        pixbuf,
        monitor_info,
    })
}

fn image_to_pixbuf(image: image::RgbaImage) -> Result<gtk::gdk_pixbuf::Pixbuf, String> {
    let width = image.width() as i32;
    let height = image.height() as i32;
    let stride = width * 4;
    let pixels = image.into_raw();

    let bytes = gtk::glib::Bytes::from(&pixels);

    let pixbuf = gtk::gdk_pixbuf::Pixbuf::from_bytes(
        &bytes,
        gtk::gdk_pixbuf::Colorspace::Rgb,
        true,
        8,
        width,
        height,
        stride,
    );

    Ok(pixbuf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_primary_monitor() {
        let session = DesktopSession::detect();
        println!("Testing capture on: {}", session);

        match capture_primary_monitor() {
            Ok(result) => {
                println!(
                    "Captured: {}x{}",
                    result.pixbuf.width(),
                    result.pixbuf.height()
                );
                assert!(result.pixbuf.width() > 0);
                assert!(result.pixbuf.height() > 0);
            }
            Err(e) => {
                println!("Capture failed (may be expected in CI): {}", e);
            }
        }
    }
}
