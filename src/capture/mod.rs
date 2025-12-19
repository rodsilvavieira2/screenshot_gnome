//! Capture module for screen and window capture functionality
//!
//! This module provides abstractions over the xcap library for capturing
//! screens and windows in a GTK-friendly way.
//! Compatible with xcap version 0.0.14.

pub mod screen;
pub mod window;

// Re-export only the items that are actually used by main.rs
pub use screen::capture_primary_monitor;
pub use window::{capture_window_by_index, list_capturable_windows};

// The following are available via the submodules for future use:
// - screen::CaptureResult, screen::MonitorInfo
// - screen::get_all_monitors, screen::get_primary_monitor, screen::get_monitor_at_point
// - screen::capture_monitor_by_id, screen::capture_monitor_by_name
// - screen::capture_monitor_at_point, screen::capture_all_monitors
// - window::WindowInfo, window::WindowCaptureResult, window::WindowCaptureError
// - window::list_all_windows, window::capture_window_by_id
