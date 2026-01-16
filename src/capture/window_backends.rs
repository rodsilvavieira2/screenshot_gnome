//! Window listing backends for different desktop environments.
//!
//! This module provides specialized window listing implementations for:
//! - Hyprland (via hyprctl)
//! - Sway (via swaymsg)
//! - GNOME Wayland (via D-Bus introspection or gdbus)
//! - KDE Wayland (via D-Bus/kdotool)
//! - X11 (via xcap)
//!
//! Each backend returns a unified `WindowInfo` structure.

use super::desktop::{DesktopSession, WindowListBackend};
use super::window::{WindowCaptureError, WindowInfo};
use std::process::Command;

/// Result type for window listing operations.
pub type WindowListResult = Result<Vec<WindowInfo>, WindowCaptureError>;

/// Lists windows using the appropriate backend for the current session.
pub fn list_windows_for_session(session: &DesktopSession) -> WindowListResult {
    let backend = session.window_list_backend();
    list_windows_with_backend(backend)
}

/// Lists windows using a specific backend.
pub fn list_windows_with_backend(backend: WindowListBackend) -> WindowListResult {
    match backend {
        WindowListBackend::Hyprland => list_windows_hyprland(),
        WindowListBackend::Sway => list_windows_sway(),
        WindowListBackend::GnomeWayland => list_windows_gnome_wayland(),
        WindowListBackend::KdeWayland => list_windows_kde_wayland(),
        WindowListBackend::X11 | WindowListBackend::Xcap => list_windows_xcap(),
    }
}

/// Lists windows using hyprctl (Hyprland).
fn list_windows_hyprland() -> WindowListResult {
    let output = Command::new("hyprctl")
        .args(["clients", "-j"])
        .output()
        .map_err(|e| {
            WindowCaptureError::EnumerationFailed(format!("Failed to run hyprctl: {}", e))
        })?;

    if !output.status.success() {
        return Err(WindowCaptureError::EnumerationFailed(
            "hyprctl returned non-zero exit code".to_string(),
        ));
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    parse_hyprland_json(&json_str)
}

/// Parses Hyprland's JSON output into WindowInfo structures.
fn parse_hyprland_json(json_str: &str) -> WindowListResult {
    // Simple JSON parsing without external dependencies
    // Hyprland returns an array of client objects
    let mut windows = Vec::new();

    // Basic JSON array parsing
    let trimmed = json_str.trim();
    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return Err(WindowCaptureError::EnumerationFailed(
            "Invalid JSON from hyprctl".to_string(),
        ));
    }

    // Extract objects from the array
    let content = &trimmed[1..trimmed.len() - 1];
    let mut depth = 0;
    let mut start = 0;
    let mut in_string = false;
    let mut escape_next = false;

    for (i, c) in content.chars().enumerate() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match c {
            '\\' if in_string => escape_next = true,
            '"' => in_string = !in_string,
            '{' if !in_string => {
                if depth == 0 {
                    start = i;
                }
                depth += 1;
            }
            '}' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    let obj_str = &content[start..=i];
                    if let Some(info) = parse_hyprland_client_object(obj_str) {
                        windows.push(info);
                    }
                }
            }
            _ => {}
        }
    }

    Ok(windows)
}

/// Parses a single Hyprland client object.
fn parse_hyprland_client_object(obj_str: &str) -> Option<WindowInfo> {
    let address = extract_json_hex_value(obj_str, "address")?;
    let id = u32::from_str_radix(&address.trim_start_matches("0x"), 16).unwrap_or(0);

    let pid = extract_json_number(obj_str, "pid").unwrap_or(0);
    let title = extract_json_string(obj_str, "title").unwrap_or_default();
    let class = extract_json_string(obj_str, "class").unwrap_or_default();

    // Parse position array [x, y]
    let (x, y) = extract_json_position(obj_str, "at").unwrap_or((0, 0));

    // Parse size array [w, h]
    let (width, height) = extract_json_size(obj_str, "size").unwrap_or((0, 0));

    let is_focused = extract_json_bool(obj_str, "focusHistoryID")
        .map(|v| v == 0)
        .unwrap_or(false);

    // Hyprland doesn't have explicit minimized/maximized in the same way
    let is_minimized = extract_json_bool_field(obj_str, "hidden").unwrap_or(false);
    let is_maximized = extract_json_bool_field(obj_str, "fullscreen").unwrap_or(false);

    Some(WindowInfo {
        id,
        pid,
        app_name: class,
        title,
        x,
        y,
        z: 0, // Hyprland doesn't provide z-order in the same way
        width,
        height,
        is_minimized,
        is_maximized,
        is_focused,
    })
}

/// Extracts a string value from JSON.
fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\":", key);
    let start = json.find(&pattern)? + pattern.len();
    let rest = json[start..].trim_start();

    if !rest.starts_with('"') {
        return None;
    }

    let content_start = 1;
    let mut end = content_start;
    let mut escape_next = false;
    let chars: Vec<char> = rest.chars().collect();

    while end < chars.len() {
        if escape_next {
            escape_next = false;
            end += 1;
            continue;
        }

        match chars[end] {
            '\\' => escape_next = true,
            '"' => {
                return Some(rest[content_start..end].to_string());
            }
            _ => {}
        }
        end += 1;
    }

    None
}

/// Extracts a hex string value (like "0x...") from JSON.
fn extract_json_hex_value(json: &str, key: &str) -> Option<String> {
    extract_json_string(json, key)
}

/// Extracts a numeric value from JSON.
fn extract_json_number(json: &str, key: &str) -> Option<u32> {
    let pattern = format!("\"{}\":", key);
    let start = json.find(&pattern)? + pattern.len();
    let rest = json[start..].trim_start();

    let end = rest
        .find(|c: char| !c.is_ascii_digit() && c != '-')
        .unwrap_or(rest.len());
    rest[..end].parse().ok()
}

/// Extracts a boolean-like number from JSON (0 or non-zero).
fn extract_json_bool(json: &str, key: &str) -> Option<i32> {
    let pattern = format!("\"{}\":", key);
    let start = json.find(&pattern)? + pattern.len();
    let rest = json[start..].trim_start();

    let end = rest
        .find(|c: char| !c.is_ascii_digit() && c != '-')
        .unwrap_or(rest.len());
    rest[..end].parse().ok()
}

/// Extracts a boolean field from JSON.
fn extract_json_bool_field(json: &str, key: &str) -> Option<bool> {
    let pattern = format!("\"{}\":", key);
    let start = json.find(&pattern)? + pattern.len();
    let rest = json[start..].trim_start();

    if rest.starts_with("true") {
        Some(true)
    } else if rest.starts_with("false") {
        Some(false)
    } else {
        None
    }
}

/// Extracts a position array [x, y] from JSON.
fn extract_json_position(json: &str, key: &str) -> Option<(i32, i32)> {
    let pattern = format!("\"{}\":", key);
    let start = json.find(&pattern)? + pattern.len();
    let rest = json[start..].trim_start();

    if !rest.starts_with('[') {
        return None;
    }

    let end = rest.find(']')?;
    let array_content = &rest[1..end];
    let parts: Vec<&str> = array_content.split(',').collect();

    if parts.len() >= 2 {
        let x = parts[0].trim().parse().ok()?;
        let y = parts[1].trim().parse().ok()?;
        Some((x, y))
    } else {
        None
    }
}

/// Extracts a size array [w, h] from JSON.
fn extract_json_size(json: &str, key: &str) -> Option<(u32, u32)> {
    let pattern = format!("\"{}\":", key);
    let start = json.find(&pattern)? + pattern.len();
    let rest = json[start..].trim_start();

    if !rest.starts_with('[') {
        return None;
    }

    let end = rest.find(']')?;
    let array_content = &rest[1..end];
    let parts: Vec<&str> = array_content.split(',').collect();

    if parts.len() >= 2 {
        let w = parts[0].trim().parse().ok()?;
        let h = parts[1].trim().parse().ok()?;
        Some((w, h))
    } else {
        None
    }
}

/// Lists windows using swaymsg (Sway).
fn list_windows_sway() -> WindowListResult {
    let output = Command::new("swaymsg")
        .args(["-t", "get_tree"])
        .output()
        .map_err(|e| {
            WindowCaptureError::EnumerationFailed(format!("Failed to run swaymsg: {}", e))
        })?;

    if !output.status.success() {
        return Err(WindowCaptureError::EnumerationFailed(
            "swaymsg returned non-zero exit code".to_string(),
        ));
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    parse_sway_tree(&json_str)
}

/// Parses Sway's tree JSON to extract window information.
fn parse_sway_tree(json_str: &str) -> WindowListResult {
    let mut windows = Vec::new();
    extract_sway_windows(json_str, &mut windows);
    Ok(windows)
}

/// Recursively extracts windows from Sway's tree structure.
fn extract_sway_windows(json_str: &str, windows: &mut Vec<WindowInfo>) {
    // Look for nodes with "type": "con" and a valid "pid"
    // This is a simplified parser - in production you'd want a proper JSON library

    let mut search_pos = 0;

    while let Some(start) = json_str[search_pos..].find("\"pid\":") {
        let abs_pos = search_pos + start;

        // Find the containing object boundaries
        if let Some(obj_start) = find_object_start(json_str, abs_pos) {
            if let Some(obj_end) = find_object_end(json_str, obj_start) {
                let obj_str = &json_str[obj_start..=obj_end];

                // Only process if it's a window (has app_id or window_properties)
                if obj_str.contains("\"app_id\"") || obj_str.contains("\"window_properties\"") {
                    if let Some(info) = parse_sway_node(obj_str) {
                        // Avoid duplicates
                        if !windows.iter().any(|w| w.id == info.id) {
                            windows.push(info);
                        }
                    }
                }

                search_pos = obj_end + 1;
                continue;
            }
        }

        search_pos = abs_pos + 6;
    }
}

/// Finds the start of a JSON object containing the given position.
fn find_object_start(json: &str, pos: usize) -> Option<usize> {
    let bytes = json.as_bytes();
    let mut depth = 0;

    for i in (0..pos).rev() {
        match bytes[i] {
            b'}' => depth += 1,
            b'{' => {
                if depth == 0 {
                    return Some(i);
                }
                depth -= 1;
            }
            _ => {}
        }
    }

    None
}

/// Finds the end of a JSON object starting at the given position.
fn find_object_end(json: &str, start: usize) -> Option<usize> {
    let bytes = json.as_bytes();
    let mut depth = 0;
    let mut in_string = false;
    let mut escape_next = false;

    for (i, &byte) in bytes[start..].iter().enumerate() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match byte {
            b'\\' if in_string => escape_next = true,
            b'"' => in_string = !in_string,
            b'{' if !in_string => depth += 1,
            b'}' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    return Some(start + i);
                }
            }
            _ => {}
        }
    }

    None
}

/// Parses a Sway node object into WindowInfo.
fn parse_sway_node(obj_str: &str) -> Option<WindowInfo> {
    let id = extract_json_number(obj_str, "id")?;
    let pid = extract_json_number(obj_str, "pid").unwrap_or(0);

    // Sway uses "name" for the title
    let title = extract_json_string(obj_str, "name").unwrap_or_default();

    // app_id is used for Wayland native apps
    let app_name = extract_json_string(obj_str, "app_id").unwrap_or_else(|| {
        // For XWayland apps, try window_properties.class
        extract_json_string(obj_str, "class").unwrap_or_default()
    });

    // Parse rect object for position and size
    let (x, y, width, height) = parse_sway_rect(obj_str).unwrap_or((0, 0, 0, 0));

    let is_focused = extract_json_bool_field(obj_str, "focused").unwrap_or(false);
    let is_maximized = extract_json_bool_field(obj_str, "fullscreen_mode")
        .map(|_| true)
        .unwrap_or(false);

    Some(WindowInfo {
        id,
        pid,
        app_name,
        title,
        x,
        y,
        z: 0,
        width,
        height,
        is_minimized: false,
        is_maximized,
        is_focused,
    })
}

/// Parses the rect object from a Sway node.
fn parse_sway_rect(obj_str: &str) -> Option<(i32, i32, u32, u32)> {
    let rect_start = obj_str.find("\"rect\":")?;
    let rest = &obj_str[rect_start..];
    let brace_start = rest.find('{')?;
    let brace_end = rest.find('}')?;
    let rect_obj = &rest[brace_start..=brace_end];

    let x = extract_json_number(rect_obj, "x").unwrap_or(0) as i32;
    let y = extract_json_number(rect_obj, "y").unwrap_or(0) as i32;
    let width = extract_json_number(rect_obj, "width").unwrap_or(0);
    let height = extract_json_number(rect_obj, "height").unwrap_or(0);

    Some((x, y, width, height))
}

/// Lists windows using GNOME Shell's D-Bus introspection (Wayland).
fn list_windows_gnome_wayland() -> WindowListResult {
    // Try using gdbus to call org.gnome.Shell.Introspect
    let output = Command::new("gdbus")
        .args([
            "call",
            "--session",
            "--dest",
            "org.gnome.Shell.Introspect",
            "--object-path",
            "/org/gnome/Shell/Introspect",
            "--method",
            "org.gnome.Shell.Introspect.GetWindows",
        ])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let result_str = String::from_utf8_lossy(&output.stdout);
            parse_gnome_introspect_output(&result_str)
        }
        _ => {
            // Fallback to xcap if GNOME introspection is not available
            eprintln!("GNOME Shell Introspect not available, falling back to xcap");
            list_windows_xcap()
        }
    }
}

/// Parses GNOME Shell Introspect output.
fn parse_gnome_introspect_output(output: &str) -> WindowListResult {
    // GNOME Shell returns a GVariant, which has a complex format
    // We'll do basic parsing here
    let mut windows = Vec::new();

    // The output format is like: ({uint64 id: {...}, ...},)
    // This is a simplified parser

    let mut search_pos = 0;
    let mut window_id: u32 = 1; // GNOME uses uint64 IDs, we'll use sequential u32s

    while let Some(start) = output[search_pos..].find("'wm-class':") {
        let abs_pos = search_pos + start;

        // Extract wm-class
        let wm_class =
            extract_gvariant_string(&output[abs_pos..], "'wm-class':").unwrap_or_default();

        // Extract title
        let title = if let Some(title_pos) = output[search_pos..].find("'title':") {
            extract_gvariant_string(&output[search_pos + title_pos..], "'title':")
                .unwrap_or_default()
        } else {
            String::new()
        };

        // Extract pid if available
        let pid = if let Some(pid_pos) = output[search_pos..].find("'pid':") {
            extract_gvariant_number(&output[search_pos + pid_pos..]).unwrap_or(0)
        } else {
            0
        };

        // Extract dimensions if available
        let (width, height) = extract_gnome_dimensions(&output[search_pos..]).unwrap_or((0, 0));

        windows.push(WindowInfo {
            id: window_id,
            pid,
            app_name: wm_class,
            title,
            x: 0,
            y: 0,
            z: 0,
            width,
            height,
            is_minimized: false,
            is_maximized: false,
            is_focused: false,
        });

        window_id += 1;
        search_pos = abs_pos + 10;
    }

    if windows.is_empty() {
        // Fallback to xcap
        list_windows_xcap()
    } else {
        Ok(windows)
    }
}

/// Extracts a string from GVariant format.
fn extract_gvariant_string(text: &str, prefix: &str) -> Option<String> {
    let start = text.find(prefix)? + prefix.len();
    let rest = text[start..].trim_start();

    // GVariant strings can be 'value' or <'value'>
    let quote_char = if rest.starts_with('<') {
        rest.find('\'')?;
        '\''
    } else if rest.starts_with('\'') {
        '\''
    } else {
        return None;
    };

    let content_start = rest.find(quote_char)? + 1;
    let content = &rest[content_start..];
    let end = content.find(quote_char)?;

    Some(content[..end].to_string())
}

/// Extracts a number from GVariant format.
fn extract_gvariant_number(text: &str) -> Option<u32> {
    let start = text.find("'pid':")? + 6;
    let rest = text[start..].trim_start();

    // Skip type annotation if present (like "uint32 123")
    let number_part = if rest.starts_with('<') {
        &rest[1..]
    } else {
        rest
    };

    // Skip type prefix like "uint32 "
    let number_str = number_part
        .split_whitespace()
        .find(|s| s.chars().all(|c| c.is_ascii_digit()))?;

    number_str.parse().ok()
}

/// Extracts window dimensions from GNOME output.
fn extract_gnome_dimensions(text: &str) -> Option<(u32, u32)> {
    let width = if let Some(pos) = text.find("'width':") {
        extract_gvariant_dimension(&text[pos..]).unwrap_or(0)
    } else {
        0
    };

    let height = if let Some(pos) = text.find("'height':") {
        extract_gvariant_dimension(&text[pos..]).unwrap_or(0)
    } else {
        0
    };

    if width > 0 && height > 0 {
        Some((width, height))
    } else {
        None
    }
}

/// Extracts a dimension value from GVariant format.
fn extract_gvariant_dimension(text: &str) -> Option<u32> {
    let colon_pos = text.find(':')?;
    let rest = text[colon_pos + 1..].trim_start();

    let end = rest
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(rest.len());

    if end > 0 {
        rest[..end].parse().ok()
    } else {
        None
    }
}

/// Lists windows using KDE/KWin's D-Bus interface (Wayland).
fn list_windows_kde_wayland() -> WindowListResult {
    // Try using qdbus or gdbus to query KWin
    let output = Command::new("gdbus")
        .args([
            "call",
            "--session",
            "--dest",
            "org.kde.KWin",
            "--object-path",
            "/KWin",
            "--method",
            "org.kde.KWin.queryWindowInfo",
        ])
        .output();

    // kdotool is another option for KDE Wayland
    let kdotool_output = Command::new("kdotool")
        .args(["search", "--name", ""])
        .output();

    if let Ok(output) = kdotool_output {
        if output.status.success() {
            let result_str = String::from_utf8_lossy(&output.stdout);
            return parse_kdotool_output(&result_str);
        }
    }

    // If KDE D-Bus doesn't work, try the queryWindowInfo method
    if let Ok(output) = output {
        if output.status.success() {
            let result_str = String::from_utf8_lossy(&output.stdout);
            return parse_kde_dbus_output(&result_str);
        }
    }

    // Fallback to xcap
    eprintln!("KDE window listing not available, falling back to xcap");
    list_windows_xcap()
}

/// Parses kdotool output into WindowInfo structures.
fn parse_kdotool_output(output: &str) -> WindowListResult {
    let mut windows = Vec::new();

    for line in output.lines() {
        if let Ok(id) = line.trim().parse::<u32>() {
            // Get window details using kdotool getwindowname
            let title = Command::new("kdotool")
                .args(["getwindowname", &id.to_string()])
                .output()
                .ok()
                .and_then(|o| {
                    if o.status.success() {
                        Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or_default();

            windows.push(WindowInfo {
                id,
                pid: 0,
                app_name: String::new(),
                title,
                x: 0,
                y: 0,
                z: 0,
                width: 0,
                height: 0,
                is_minimized: false,
                is_maximized: false,
                is_focused: false,
            });
        }
    }

    if windows.is_empty() {
        list_windows_xcap()
    } else {
        Ok(windows)
    }
}

/// Parses KDE D-Bus output.
fn parse_kde_dbus_output(_output: &str) -> WindowListResult {
    // KDE's D-Bus interface is complex, fallback to xcap for now
    list_windows_xcap()
}

/// Lists windows using xcap (fallback for X11 and unsupported environments).
fn list_windows_xcap() -> WindowListResult {
    use xcap::Window;

    let windows = Window::all().map_err(|e| {
        WindowCaptureError::EnumerationFailed(format!("xcap failed to list windows: {}", e))
    })?;

    let mut window_infos = Vec::new();

    for window in &windows {
        let info = WindowInfo {
            id: window.id().unwrap_or(0),
            pid: window.pid().unwrap_or(0),
            app_name: window.app_name().unwrap_or_default(),
            title: window.title().unwrap_or_default(),
            x: window.x().unwrap_or(0),
            y: window.y().unwrap_or(0),
            z: window.z().unwrap_or(0),
            width: window.width().unwrap_or(0),
            height: window.height().unwrap_or(0),
            is_minimized: window.is_minimized().unwrap_or(false),
            is_maximized: window.is_maximized().unwrap_or(false),
            is_focused: window.is_focused().unwrap_or(false),
        };

        window_infos.push(info);
    }

    Ok(window_infos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hyprland_json() {
        let json = r#"[
            {
                "address": "0x12345678",
                "mapped": true,
                "hidden": false,
                "at": [100, 200],
                "size": [800, 600],
                "workspace": {"id": 1, "name": "1"},
                "floating": false,
                "monitor": 0,
                "class": "firefox",
                "title": "Mozilla Firefox",
                "initialClass": "firefox",
                "initialTitle": "Mozilla Firefox",
                "pid": 1234,
                "xwayland": false,
                "pinned": false,
                "fullscreen": false,
                "fullscreenMode": 0,
                "fakeFullscreen": false,
                "grouped": [],
                "swallowing": "0x0",
                "focusHistoryID": 0
            }
        ]"#;

        let result = parse_hyprland_json(json);
        assert!(result.is_ok());
        let windows = result.unwrap();
        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].title, "Mozilla Firefox");
        assert_eq!(windows[0].app_name, "firefox");
        assert_eq!(windows[0].pid, 1234);
    }

    #[test]
    fn test_list_windows_for_current_session() {
        let session = DesktopSession::detect();
        println!("Testing window list for: {}", session);
        println!("Using backend: {}", session.window_list_backend());

        // Just verify it doesn't panic
        let result = list_windows_for_session(&session);
        match result {
            Ok(windows) => {
                println!("Found {} windows", windows.len());
                for window in &windows {
                    println!("  - {} ({})", window.title, window.app_name);
                }
            }
            Err(e) => {
                println!("Error listing windows: {}", e);
            }
        }
    }
}
