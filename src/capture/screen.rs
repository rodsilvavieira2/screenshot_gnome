#![allow(dead_code)]

use gtk4 as gtk;
use std::process::Command;
use xcap::Monitor;

use super::desktop::{DesktopEnvironment, DesktopSession, DisplayServer};

#[derive(Debug, Clone)]
pub struct MonitorInfo {
    pub id: u32,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
    pub scale_factor: f32,
    pub rotation: f32,
    pub frequency: f32,
    pub is_builtin: bool,
}

impl MonitorInfo {
    fn from_xcap(monitor: &Monitor) -> Result<Self, String> {
        Ok(Self {
            id: monitor.id().map_err(|e| e.to_string())?,
            name: monitor.name().map_err(|e| e.to_string())?,
            x: monitor.x().map_err(|e| e.to_string())?,
            y: monitor.y().map_err(|e| e.to_string())?,
            width: monitor.width().map_err(|e| e.to_string())?,
            height: monitor.height().map_err(|e| e.to_string())?,
            is_primary: monitor.is_primary().map_err(|e| e.to_string())?,
            scale_factor: monitor.scale_factor().map_err(|e| e.to_string())?,
            rotation: monitor.rotation().map_err(|e| e.to_string())?,
            frequency: monitor.frequency().map_err(|e| e.to_string())?,
            is_builtin: monitor.is_builtin().map_err(|e| e.to_string())?,
        })
    }

    /// Create a default MonitorInfo for Wayland when we can't get detailed info
    fn default_wayland() -> Self {
        Self {
            id: 0,
            name: "Wayland Screen".to_string(),
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
            is_primary: true,
            scale_factor: 1.0,
            rotation: 0.0,
            frequency: 60.0,
            is_builtin: false,
        }
    }
}

pub struct CaptureResult {
    pub pixbuf: gtk::gdk_pixbuf::Pixbuf,
    pub monitor_info: MonitorInfo,
}

pub fn get_all_monitors() -> Result<Vec<MonitorInfo>, String> {
    let monitors = Monitor::all().map_err(|e| format!("Failed to get monitors: {}", e))?;

    let mut infos = Vec::new();
    for monitor in &monitors {
        match MonitorInfo::from_xcap(monitor) {
            Ok(info) => infos.push(info),
            Err(e) => eprintln!("Warning: Failed to get info for a monitor: {}", e),
        }
    }

    if infos.is_empty() {
        Err("No monitors found".to_string())
    } else {
        Ok(infos)
    }
}

pub fn get_primary_monitor() -> Result<MonitorInfo, String> {
    let monitors = Monitor::all().map_err(|e| format!("Failed to get monitors: {}", e))?;

    for monitor in &monitors {
        if monitor.is_primary().unwrap_or(false) {
            return MonitorInfo::from_xcap(monitor);
        }
    }

    monitors
        .first()
        .ok_or_else(|| "No monitors found".to_string())
        .and_then(MonitorInfo::from_xcap)
}

pub fn get_monitor_at_point(x: i32, y: i32) -> Result<MonitorInfo, String> {
    let monitor =
        Monitor::from_point(x, y).map_err(|e| format!("Failed to get monitor at point: {}", e))?;

    MonitorInfo::from_xcap(&monitor)
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

    // Get monitor info from pixbuf dimensions
    let monitor_info = MonitorInfo {
        width: pixbuf.width() as u32,
        height: pixbuf.height() as u32,
        ..MonitorInfo::default_wayland()
    };

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

    let monitor_info = MonitorInfo {
        width: pixbuf.width() as u32,
        height: pixbuf.height() as u32,
        ..MonitorInfo::default_wayland()
    };

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

    let monitor_info = MonitorInfo {
        width: pixbuf.width() as u32,
        height: pixbuf.height() as u32,
        ..MonitorInfo::default_wayland()
    };

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

pub fn capture_monitor_by_id(monitor_id: u32) -> Result<CaptureResult, String> {
    let monitors = Monitor::all().map_err(|e| format!("Failed to get monitors: {}", e))?;

    let monitor = monitors
        .iter()
        .find(|m| m.id().ok() == Some(monitor_id))
        .ok_or_else(|| format!("Monitor with ID {} not found", monitor_id))?;

    capture_monitor_internal(monitor)
}

pub fn capture_monitor_by_name(name: &str) -> Result<CaptureResult, String> {
    let monitors = Monitor::all().map_err(|e| format!("Failed to get monitors: {}", e))?;

    let monitor = monitors
        .iter()
        .find(|m| m.name().ok().as_deref() == Some(name))
        .ok_or_else(|| format!("Monitor '{}' not found", name))?;

    capture_monitor_internal(monitor)
}

pub fn capture_monitor_at_point(x: i32, y: i32) -> Result<CaptureResult, String> {
    let monitor =
        Monitor::from_point(x, y).map_err(|e| format!("Failed to get monitor at point: {}", e))?;

    capture_monitor_internal(&monitor)
}

pub fn capture_all_monitors() -> Result<Vec<CaptureResult>, String> {
    let monitors = Monitor::all().map_err(|e| format!("Failed to get monitors: {}", e))?;

    let mut results = Vec::new();

    for monitor in &monitors {
        match capture_monitor_internal(monitor) {
            Ok(result) => results.push(result),
            Err(e) => eprintln!("Failed to capture monitor: {}", e),
        }
    }

    if results.is_empty() {
        Err("Failed to capture any monitors".to_string())
    } else {
        Ok(results)
    }
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
    fn test_get_all_monitors() {
        if let Ok(monitors) = get_all_monitors() {
            assert!(!monitors.is_empty());
            for monitor in &monitors {
                println!(
                    "Monitor: {} ({}x{}) at ({}, {})",
                    monitor.name, monitor.width, monitor.height, monitor.x, monitor.y
                );
            }
        }
    }

    #[test]
    fn test_get_primary_monitor() {
        if let Ok(monitor) = get_primary_monitor() {
            println!("Primary monitor: {}", monitor.name);
            assert!(monitor.width > 0);
            assert!(monitor.height > 0);
        }
    }

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
