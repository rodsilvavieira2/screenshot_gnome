//! Screen capture module using xcap library
//!
//! This module provides functionality to capture monitors/screens
//! following the xcap library patterns for version 0.4+.
//! All xcap methods now return XCapResult<T>.

#![allow(dead_code)]

use gtk4 as gtk;
use xcap::Monitor;

/// Information about a monitor
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
    /// Create MonitorInfo from xcap Monitor
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
}

/// Result of a screen capture operation
pub struct CaptureResult {
    pub pixbuf: gtk::gdk_pixbuf::Pixbuf,
    pub monitor_info: MonitorInfo,
}

/// Get all available monitors
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

/// Get the primary monitor
pub fn get_primary_monitor() -> Result<MonitorInfo, String> {
    let monitors = Monitor::all().map_err(|e| format!("Failed to get monitors: {}", e))?;

    for monitor in &monitors {
        if monitor.is_primary().unwrap_or(false) {
            return MonitorInfo::from_xcap(monitor);
        }
    }

    // Fallback to first monitor
    monitors
        .first()
        .ok_or_else(|| "No monitors found".to_string())
        .and_then(MonitorInfo::from_xcap)
}

/// Get monitor at specific point
pub fn get_monitor_at_point(x: i32, y: i32) -> Result<MonitorInfo, String> {
    let monitor =
        Monitor::from_point(x, y).map_err(|e| format!("Failed to get monitor at point: {}", e))?;

    MonitorInfo::from_xcap(&monitor)
}

/// Capture the primary monitor
pub fn capture_primary_monitor() -> Result<CaptureResult, String> {
    let monitors = Monitor::all().map_err(|e| format!("Failed to get monitors: {}", e))?;

    // Find primary monitor
    let monitor = monitors
        .iter()
        .find(|m| m.is_primary().unwrap_or(false))
        .or(monitors.first())
        .ok_or("No monitors available")?;

    capture_monitor_internal(monitor)
}

/// Capture a specific monitor by ID
pub fn capture_monitor_by_id(monitor_id: u32) -> Result<CaptureResult, String> {
    let monitors = Monitor::all().map_err(|e| format!("Failed to get monitors: {}", e))?;

    let monitor = monitors
        .iter()
        .find(|m| m.id().ok() == Some(monitor_id))
        .ok_or_else(|| format!("Monitor with ID {} not found", monitor_id))?;

    capture_monitor_internal(monitor)
}

/// Capture a specific monitor by name
pub fn capture_monitor_by_name(name: &str) -> Result<CaptureResult, String> {
    let monitors = Monitor::all().map_err(|e| format!("Failed to get monitors: {}", e))?;

    let monitor = monitors
        .iter()
        .find(|m| m.name().ok().as_deref() == Some(name))
        .ok_or_else(|| format!("Monitor '{}' not found", name))?;

    capture_monitor_internal(monitor)
}

/// Capture monitor at a specific point
pub fn capture_monitor_at_point(x: i32, y: i32) -> Result<CaptureResult, String> {
    let monitor =
        Monitor::from_point(x, y).map_err(|e| format!("Failed to get monitor at point: {}", e))?;

    capture_monitor_internal(&monitor)
}

/// Capture all monitors and return results for each
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

/// Internal function to capture a monitor
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

/// Convert xcap image (RgbaImage) to GDK Pixbuf
fn image_to_pixbuf(image: image::RgbaImage) -> Result<gtk::gdk_pixbuf::Pixbuf, String> {
    let width = image.width() as i32;
    let height = image.height() as i32;
    let stride = width * 4; // RGBA = 4 bytes per pixel
    let pixels = image.into_raw();

    let bytes = gtk::glib::Bytes::from(&pixels);

    let pixbuf = gtk::gdk_pixbuf::Pixbuf::from_bytes(
        &bytes,
        gtk::gdk_pixbuf::Colorspace::Rgb,
        true, // has_alpha
        8,    // bits_per_sample
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
        // This test may fail in CI environments without display
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
        // This test may fail in CI environments without display
        if let Ok(monitor) = get_primary_monitor() {
            println!("Primary monitor: {}", monitor.name);
            assert!(monitor.width > 0);
            assert!(monitor.height > 0);
        }
    }
}
