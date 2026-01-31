use super::desktop::{DesktopSession, WindowListBackend};
use super::window::{WindowCaptureError, WindowCaptureResult, WindowInfo};
use gtk4::gdk_pixbuf::{Colorspace, Pixbuf};
use gtk4::glib;
use log::{debug, warn};
use std::process::Command;

pub type WindowListResult = Result<Vec<WindowInfo>, WindowCaptureError>;

pub type WindowCaptureBackendResult = Result<WindowCaptureResult, WindowCaptureError>;

pub fn list_windows_for_session(session: &DesktopSession) -> WindowListResult {
    let backend = session.window_list_backend();
    list_windows_with_backend(backend)
}

pub fn list_windows_with_backend(backend: WindowListBackend) -> WindowListResult {
    debug!("Listing windows with backend: {:?}", backend);
    match backend {
        WindowListBackend::Hyprland => list_windows_hyprland(),
        WindowListBackend::Sway => list_windows_sway(),
        WindowListBackend::GnomeWayland => list_windows_gnome_wayland(),
        WindowListBackend::KdeWayland => list_windows_kde_wayland(),
        WindowListBackend::X11 | WindowListBackend::Xcap => list_windows_xcap(),
    }
}

pub fn capture_window_for_session(
    session: &DesktopSession,
    window_info: &WindowInfo,
) -> WindowCaptureBackendResult {
    let backend = session.window_list_backend();
    capture_window_with_backend(backend, window_info)
}

pub fn capture_window_with_backend(
    backend: WindowListBackend,
    window_info: &WindowInfo,
) -> WindowCaptureBackendResult {
    debug!(
        "Capturing window with backend: {:?}, window: {}",
        backend,
        window_info.display_label()
    );
    match backend {
        WindowListBackend::Hyprland => capture_window_hyprland(window_info),
        WindowListBackend::Sway => capture_window_sway(window_info),
        WindowListBackend::GnomeWayland => capture_window_gnome_wayland(window_info),
        WindowListBackend::KdeWayland => capture_window_kde_wayland(window_info),
        WindowListBackend::X11 | WindowListBackend::Xcap => capture_window_xcap(window_info),
    }
}

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

fn parse_hyprland_json(json_str: &str) -> WindowListResult {
    let mut windows = Vec::new();

    let trimmed = json_str.trim();
    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return Err(WindowCaptureError::EnumerationFailed(
            "Invalid JSON from hyprctl".to_string(),
        ));
    }

    let content = &trimmed[1..trimmed.len() - 1];
    let mut depth = 0;
    let mut start = 0;
    let mut in_string = false;
    let mut escape_next = false;

    for (i, c) in content.char_indices() {
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

fn parse_hyprland_client_object(obj_str: &str) -> Option<WindowInfo> {
    let address = extract_json_hex_value(obj_str, "address")?;
    let id = u32::from_str_radix(address.trim_start_matches("0x"), 16).unwrap_or(0);

    let pid = extract_json_number(obj_str, "pid").unwrap_or(0);
    let title = extract_json_string(obj_str, "title").unwrap_or_default();
    let class = extract_json_string(obj_str, "class").unwrap_or_default();

    let (x, y) = extract_json_position(obj_str, "at").unwrap_or((0, 0));

    let (width, height) = extract_json_size(obj_str, "size").unwrap_or((0, 0));

    let is_focused = extract_json_bool(obj_str, "focusHistoryID")
        .map(|v| v == 0)
        .unwrap_or(false);

    let is_minimized = extract_json_bool_field(obj_str, "hidden").unwrap_or(false);
    let is_maximized = extract_json_bool_field(obj_str, "fullscreen").unwrap_or(false);

    Some(WindowInfo {
        id,
        pid,
        app_name: class,
        title,
        x,
        y,
        z: 0,
        width,
        height,
        is_minimized,
        is_maximized,
        is_focused,
    })
}

fn capture_window_hyprland(window_info: &WindowInfo) -> WindowCaptureBackendResult {
    let geometry = format!(
        "{},{} {}x{}",
        window_info.x, window_info.y, window_info.width, window_info.height
    );

    let temp_path = format!("/tmp/screenshot_gnome_{}.png", std::process::id());

    let output = Command::new("grim")
        .args(["-g", &geometry, &temp_path])
        .output()
        .map_err(|e| WindowCaptureError::CaptureFailed(format!("Failed to run grim: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(WindowCaptureError::CaptureFailed(format!(
            "grim failed: {}",
            stderr
        )));
    }

    let pixbuf = load_pixbuf_from_file(&temp_path)?;

    let _ = std::fs::remove_file(&temp_path);

    Ok(WindowCaptureResult {
        pixbuf,
        window_info: window_info.clone(),
    })
}

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

fn extract_json_hex_value(json: &str, key: &str) -> Option<String> {
    extract_json_string(json, key)
}

fn extract_json_number(json: &str, key: &str) -> Option<u32> {
    let pattern = format!("\"{}\":", key);
    let start = json.find(&pattern)? + pattern.len();
    let rest = json[start..].trim_start();

    let end = rest
        .find(|c: char| !c.is_ascii_digit() && c != '-')
        .unwrap_or(rest.len());
    rest[..end].parse().ok()
}

fn extract_json_bool(json: &str, key: &str) -> Option<i32> {
    let pattern = format!("\"{}\":", key);
    let start = json.find(&pattern)? + pattern.len();
    let rest = json[start..].trim_start();

    let end = rest
        .find(|c: char| !c.is_ascii_digit() && c != '-')
        .unwrap_or(rest.len());
    rest[..end].parse().ok()
}

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

fn parse_sway_tree(json_str: &str) -> WindowListResult {
    let mut windows = Vec::new();
    extract_sway_windows(json_str, &mut windows);
    Ok(windows)
}

fn extract_sway_windows(json_str: &str, windows: &mut Vec<WindowInfo>) {
    let mut search_pos = 0;

    while let Some(start) = json_str[search_pos..].find("\"pid\":") {
        let abs_pos = search_pos + start;

        if let Some(obj_start) = find_object_start(json_str, abs_pos) {
            if let Some(obj_end) = find_object_end(json_str, obj_start) {
                let obj_str = &json_str[obj_start..=obj_end];

                if obj_str.contains("\"app_id\"") || obj_str.contains("\"window_properties\"") {
                    if let Some(info) = parse_sway_node(obj_str) {
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

fn parse_sway_node(obj_str: &str) -> Option<WindowInfo> {
    let id = extract_json_number(obj_str, "id")?;
    let pid = extract_json_number(obj_str, "pid").unwrap_or(0);

    let title = extract_json_string(obj_str, "name").unwrap_or_default();

    let app_name = extract_json_string(obj_str, "app_id")
        .unwrap_or_else(|| extract_json_string(obj_str, "class").unwrap_or_default());

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

fn capture_window_sway(window_info: &WindowInfo) -> WindowCaptureBackendResult {
    let geometry = format!(
        "{},{} {}x{}",
        window_info.x, window_info.y, window_info.width, window_info.height
    );

    let temp_path = format!("/tmp/screenshot_gnome_{}.png", std::process::id());

    let output = Command::new("grim")
        .args(["-g", &geometry, &temp_path])
        .output()
        .map_err(|e| WindowCaptureError::CaptureFailed(format!("Failed to run grim: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(WindowCaptureError::CaptureFailed(format!(
            "grim failed: {}",
            stderr
        )));
    }

    let pixbuf = load_pixbuf_from_file(&temp_path)?;
    let _ = std::fs::remove_file(&temp_path);

    Ok(WindowCaptureResult {
        pixbuf,
        window_info: window_info.clone(),
    })
}

fn list_windows_gnome_wayland() -> WindowListResult {
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
            warn!("GNOME Shell Introspect not available, falling back to xcap");
            list_windows_xcap()
        }
    }
}

fn parse_gnome_introspect_output(output: &str) -> WindowListResult {
    let mut windows = Vec::new();

    let mut search_pos = 0;
    let mut window_id: u32 = 1;

    while let Some(start) = output[search_pos..].find("'wm-class':") {
        let abs_pos = search_pos + start;

        let wm_class =
            extract_gvariant_string(&output[abs_pos..], "'wm-class':").unwrap_or_default();

        let title = if let Some(title_pos) = output[search_pos..].find("'title':") {
            extract_gvariant_string(&output[search_pos + title_pos..], "'title':")
                .unwrap_or_default()
        } else {
            String::new()
        };

        let pid = if let Some(pid_pos) = output[search_pos..].find("'pid':") {
            extract_gvariant_number(&output[search_pos + pid_pos..]).unwrap_or(0)
        } else {
            0
        };

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
        list_windows_xcap()
    } else {
        Ok(windows)
    }
}

fn extract_gvariant_string(text: &str, prefix: &str) -> Option<String> {
    let start = text.find(prefix)? + prefix.len();
    let rest = text[start..].trim_start();

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

fn extract_gvariant_number(text: &str) -> Option<u32> {
    let start = text.find("'pid':")? + 6;
    let rest = text[start..].trim_start();

    let number_part = rest.strip_prefix('<').unwrap_or(rest);

    let number_str = number_part
        .split_whitespace()
        .find(|s| s.chars().all(|c| c.is_ascii_digit()))?;

    number_str.parse().ok()
}

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

fn capture_window_gnome_wayland(window_info: &WindowInfo) -> WindowCaptureBackendResult {
    let temp_path = format!("/tmp/screenshot_gnome_{}.png", std::process::id());

    let portal_result = Command::new("gdbus")
        .args([
            "call",
            "--session",
            "--dest",
            "org.gnome.Shell.Screenshot",
            "--object-path",
            "/org/gnome/Shell/Screenshot",
            "--method",
            "org.gnome.Shell.Screenshot.ScreenshotWindow",
            "true",
            "true",
            &temp_path,
        ])
        .output();

    if let Ok(output) = portal_result {
        if output.status.success() {
            if let Ok(pixbuf) = load_pixbuf_from_file(&temp_path) {
                let _ = std::fs::remove_file(&temp_path);
                return Ok(WindowCaptureResult {
                    pixbuf,
                    window_info: window_info.clone(),
                });
            }
        }
    }

    let geometry = format!(
        "{},{} {}x{}",
        window_info.x, window_info.y, window_info.width, window_info.height
    );

    let grim_result = Command::new("grim")
        .args(["-g", &geometry, &temp_path])
        .output();

    if let Ok(output) = grim_result {
        if output.status.success() {
            if let Ok(pixbuf) = load_pixbuf_from_file(&temp_path) {
                let _ = std::fs::remove_file(&temp_path);
                return Ok(WindowCaptureResult {
                    pixbuf,
                    window_info: window_info.clone(),
                });
            }
        }
    }

    let gnome_result = Command::new("gnome-screenshot")
        .args(["-f", &temp_path])
        .output();

    if let Ok(output) = gnome_result {
        if output.status.success() {
            if let Ok(full_pixbuf) = load_pixbuf_from_file(&temp_path) {
                let _ = std::fs::remove_file(&temp_path);

                if let Some(cropped) = crop_pixbuf(
                    &full_pixbuf,
                    window_info.x,
                    window_info.y,
                    window_info.width as i32,
                    window_info.height as i32,
                ) {
                    return Ok(WindowCaptureResult {
                        pixbuf: cropped,
                        window_info: window_info.clone(),
                    });
                }
            }
        }
    }

    capture_window_xcap(window_info)
}

fn list_windows_kde_wayland() -> WindowListResult {
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

    let kdotool_output = Command::new("kdotool")
        .args(["search", "--name", ""])
        .output();

    if let Ok(output) = kdotool_output {
        if output.status.success() {
            let result_str = String::from_utf8_lossy(&output.stdout);
            return parse_kdotool_output(&result_str);
        }
    }

    if let Ok(output) = output {
        if output.status.success() {
            let result_str = String::from_utf8_lossy(&output.stdout);
            return parse_kde_dbus_output(&result_str);
        }
    }

    warn!("KDE window listing not available, falling back to xcap");
    list_windows_xcap()
}

fn parse_kdotool_output(output: &str) -> WindowListResult {
    let mut windows = Vec::new();

    for line in output.lines() {
        if let Ok(id) = line.trim().parse::<u32>() {
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

fn parse_kde_dbus_output(_output: &str) -> WindowListResult {
    list_windows_xcap()
}

fn capture_window_kde_wayland(window_info: &WindowInfo) -> WindowCaptureBackendResult {
    let temp_path = format!("/tmp/screenshot_gnome_{}.png", std::process::id());

    let spectacle_result = Command::new("spectacle")
        .args(["-r", "-b", "-n", "-o", &temp_path])
        .output();

    let spectacle_window = Command::new("spectacle")
        .args(["-a", "-b", "-n", "-o", &temp_path])
        .output();

    if let Ok(output) = spectacle_window {
        if output.status.success() {
            if let Ok(pixbuf) = load_pixbuf_from_file(&temp_path) {
                let _ = std::fs::remove_file(&temp_path);
                return Ok(WindowCaptureResult {
                    pixbuf,
                    window_info: window_info.clone(),
                });
            }
        }
    }

    let grim_geometry = format!(
        "{},{} {}x{}",
        window_info.x, window_info.y, window_info.width, window_info.height
    );

    let grim_result = Command::new("grim")
        .args(["-g", &grim_geometry, &temp_path])
        .output();

    if let Ok(output) = grim_result {
        if output.status.success() {
            if let Ok(pixbuf) = load_pixbuf_from_file(&temp_path) {
                let _ = std::fs::remove_file(&temp_path);
                return Ok(WindowCaptureResult {
                    pixbuf,
                    window_info: window_info.clone(),
                });
            }
        }
    }

    let _ = spectacle_result;
    capture_window_xcap(window_info)
}

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

fn capture_window_xcap(window_info: &WindowInfo) -> WindowCaptureBackendResult {
    use xcap::Window;

    let windows = Window::all().map_err(|e| {
        WindowCaptureError::EnumerationFailed(format!("xcap failed to list windows: {}", e))
    })?;

    let window = windows
        .iter()
        .find(|w| w.id().ok() == Some(window_info.id))
        .or_else(|| {
            windows.iter().find(|w| {
                w.title().ok().as_deref() == Some(&window_info.title)
                    && w.app_name().ok().as_deref() == Some(&window_info.app_name)
            })
        })
        .or_else(|| {
            windows.iter().find(|w| {
                w.x().ok() == Some(window_info.x)
                    && w.y().ok() == Some(window_info.y)
                    && w.width().ok() == Some(window_info.width)
                    && w.height().ok() == Some(window_info.height)
            })
        });

    let window = window.ok_or(WindowCaptureError::WindowNotFound)?;

    if window.is_minimized().unwrap_or(false) {
        return Err(WindowCaptureError::WindowMinimized);
    }

    let image = window
        .capture_image()
        .map_err(|e| WindowCaptureError::CaptureFailed(e.to_string()))?;

    let pixbuf = rgba_image_to_pixbuf(image)?;

    Ok(WindowCaptureResult {
        pixbuf,
        window_info: window_info.clone(),
    })
}

fn load_pixbuf_from_file(path: &str) -> Result<Pixbuf, WindowCaptureError> {
    Pixbuf::from_file(path)
        .map_err(|e| WindowCaptureError::ConversionFailed(format!("Failed to load image: {}", e)))
}

fn crop_pixbuf(pixbuf: &Pixbuf, x: i32, y: i32, width: i32, height: i32) -> Option<Pixbuf> {
    let src_width = pixbuf.width();
    let src_height = pixbuf.height();

    let x = x.max(0).min(src_width - 1);
    let y = y.max(0).min(src_height - 1);
    let width = width.min(src_width - x);
    let height = height.min(src_height - y);

    if width <= 0 || height <= 0 {
        return None;
    }

    Some(pixbuf.new_subpixbuf(x, y, width, height))
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
