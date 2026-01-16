//! Desktop environment and display server detection module.
//!
//! This module provides functionality to detect:
//! - The current display server (Wayland or X11)
//! - The desktop environment (GNOME, KDE Plasma, Hyprland, etc.)
//!
//! This information is used to determine the appropriate method for
//! listing and capturing windows.

use std::env;
use std::process::Command;

/// The display server protocol in use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayServer {
    /// Wayland display server
    Wayland,
    /// X11/Xorg display server
    X11,
    /// Unknown or unsupported display server
    Unknown,
}

impl std::fmt::Display for DisplayServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DisplayServer::Wayland => write!(f, "Wayland"),
            DisplayServer::X11 => write!(f, "X11"),
            DisplayServer::Unknown => write!(f, "Unknown"),
        }
    }
}

/// The desktop environment in use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesktopEnvironment {
    /// GNOME desktop environment
    Gnome,
    /// KDE Plasma desktop environment
    Kde,
    /// Hyprland compositor (Wayland-only)
    Hyprland,
    /// Sway compositor (Wayland-only)
    Sway,
    /// Cinnamon desktop environment
    Cinnamon,
    /// XFCE desktop environment
    Xfce,
    /// MATE desktop environment
    Mate,
    /// Other or unknown desktop environment
    Other(Option<String>),
}

impl std::fmt::Display for DesktopEnvironment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DesktopEnvironment::Gnome => write!(f, "GNOME"),
            DesktopEnvironment::Kde => write!(f, "KDE Plasma"),
            DesktopEnvironment::Hyprland => write!(f, "Hyprland"),
            DesktopEnvironment::Sway => write!(f, "Sway"),
            DesktopEnvironment::Cinnamon => write!(f, "Cinnamon"),
            DesktopEnvironment::Xfce => write!(f, "XFCE"),
            DesktopEnvironment::Mate => write!(f, "MATE"),
            DesktopEnvironment::Other(Some(name)) => write!(f, "{}", name),
            DesktopEnvironment::Other(None) => write!(f, "Unknown"),
        }
    }
}

/// Combined desktop session information.
#[derive(Debug, Clone)]
pub struct DesktopSession {
    /// The display server protocol
    pub display_server: DisplayServer,
    /// The desktop environment
    pub desktop_environment: DesktopEnvironment,
}

impl DesktopSession {
    /// Detects the current desktop session.
    pub fn detect() -> Self {
        let display_server = detect_display_server();
        let desktop_environment = detect_desktop_environment(&display_server);

        Self {
            display_server,
            desktop_environment,
        }
    }

    /// Returns true if running on Wayland.
    pub fn is_wayland(&self) -> bool {
        self.display_server == DisplayServer::Wayland
    }

    /// Returns true if running on X11.
    pub fn is_x11(&self) -> bool {
        self.display_server == DisplayServer::X11
    }

    /// Returns true if running on GNOME.
    pub fn is_gnome(&self) -> bool {
        self.desktop_environment == DesktopEnvironment::Gnome
    }

    /// Returns true if running on KDE Plasma.
    pub fn is_kde(&self) -> bool {
        self.desktop_environment == DesktopEnvironment::Kde
    }

    /// Returns true if running on Hyprland.
    pub fn is_hyprland(&self) -> bool {
        self.desktop_environment == DesktopEnvironment::Hyprland
    }

    /// Returns true if running on Sway.
    pub fn is_sway(&self) -> bool {
        self.desktop_environment == DesktopEnvironment::Sway
    }

    /// Returns the recommended window listing backend for this session.
    pub fn window_list_backend(&self) -> WindowListBackend {
        match (&self.desktop_environment, &self.display_server) {
            (DesktopEnvironment::Hyprland, DisplayServer::Wayland) => WindowListBackend::Hyprland,
            (DesktopEnvironment::Sway, DisplayServer::Wayland) => WindowListBackend::Sway,
            (DesktopEnvironment::Gnome, DisplayServer::Wayland) => WindowListBackend::GnomeWayland,
            (DesktopEnvironment::Kde, DisplayServer::Wayland) => WindowListBackend::KdeWayland,
            (_, DisplayServer::X11) => WindowListBackend::X11,
            _ => WindowListBackend::Xcap,
        }
    }
}

impl std::fmt::Display for DesktopSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} on {}", self.desktop_environment, self.display_server)
    }
}

/// The backend to use for listing windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowListBackend {
    /// Use hyprctl for Hyprland
    Hyprland,
    /// Use swaymsg for Sway
    Sway,
    /// Use GNOME Shell D-Bus introspection
    GnomeWayland,
    /// Use KWin D-Bus interface
    KdeWayland,
    /// Use X11 APIs (via xcap or similar)
    X11,
    /// Use xcap library (fallback)
    Xcap,
}

impl std::fmt::Display for WindowListBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowListBackend::Hyprland => write!(f, "Hyprland (hyprctl)"),
            WindowListBackend::Sway => write!(f, "Sway (swaymsg)"),
            WindowListBackend::GnomeWayland => write!(f, "GNOME Wayland (D-Bus)"),
            WindowListBackend::KdeWayland => write!(f, "KDE Wayland (D-Bus)"),
            WindowListBackend::X11 => write!(f, "X11"),
            WindowListBackend::Xcap => write!(f, "xcap (fallback)"),
        }
    }
}

/// Detects the current display server.
fn detect_display_server() -> DisplayServer {
    // Check XDG_SESSION_TYPE first (most reliable on modern systems)
    if let Ok(session_type) = env::var("XDG_SESSION_TYPE") {
        match session_type.to_lowercase().as_str() {
            "wayland" => return DisplayServer::Wayland,
            "x11" => return DisplayServer::X11,
            _ => {}
        }
    }

    // Check for Wayland-specific environment variables
    if env::var("WAYLAND_DISPLAY").is_ok() {
        return DisplayServer::Wayland;
    }

    // Check for X11-specific environment variables
    if env::var("DISPLAY").is_ok() {
        // Note: DISPLAY can be set even under Wayland (for XWayland)
        // So we only use this as a fallback
        return DisplayServer::X11;
    }

    DisplayServer::Unknown
}

/// Detects the current desktop environment.
fn detect_desktop_environment(display_server: &DisplayServer) -> DesktopEnvironment {
    // Check for Hyprland first (Wayland-only compositor)
    if env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
        return DesktopEnvironment::Hyprland;
    }

    // Check for Sway (Wayland-only compositor)
    if env::var("SWAYSOCK").is_ok() {
        return DesktopEnvironment::Sway;
    }

    // Check XDG_CURRENT_DESKTOP (can contain multiple values separated by colons)
    if let Ok(current_desktop) = env::var("XDG_CURRENT_DESKTOP") {
        let desktop_lower = current_desktop.to_lowercase();

        // Check each component
        for component in desktop_lower.split(':') {
            match component.trim() {
                "gnome" | "unity" | "ubuntu" | "pop" => {
                    return DesktopEnvironment::Gnome;
                }
                "kde" | "plasma" | "kde-plasma" => {
                    return DesktopEnvironment::Kde;
                }
                "hyprland" => {
                    return DesktopEnvironment::Hyprland;
                }
                "sway" => {
                    return DesktopEnvironment::Sway;
                }
                "cinnamon" | "x-cinnamon" => {
                    return DesktopEnvironment::Cinnamon;
                }
                "xfce" | "xfce4" => {
                    return DesktopEnvironment::Xfce;
                }
                "mate" => {
                    return DesktopEnvironment::Mate;
                }
                _ => continue,
            }
        }

        // If we couldn't match but have a value, return it as Other
        if !current_desktop.is_empty() {
            return DesktopEnvironment::Other(Some(current_desktop));
        }
    }

    // Fallback: Check DESKTOP_SESSION
    if let Ok(desktop_session) = env::var("DESKTOP_SESSION") {
        let session_lower = desktop_session.to_lowercase();

        if session_lower.contains("gnome") {
            return DesktopEnvironment::Gnome;
        } else if session_lower.contains("plasma") || session_lower.contains("kde") {
            return DesktopEnvironment::Kde;
        } else if session_lower.contains("cinnamon") {
            return DesktopEnvironment::Cinnamon;
        } else if session_lower.contains("xfce") {
            return DesktopEnvironment::Xfce;
        } else if session_lower.contains("mate") {
            return DesktopEnvironment::Mate;
        }
    }

    // Fallback: Check for KDE-specific environment variable
    if env::var("KDE_FULL_SESSION").is_ok() {
        return DesktopEnvironment::Kde;
    }

    // Fallback: Check GNOME_DESKTOP_SESSION_ID (older systems)
    if env::var("GNOME_DESKTOP_SESSION_ID").is_ok() {
        return DesktopEnvironment::Gnome;
    }

    // Additional check for Hyprland by testing hyprctl
    if *display_server == DisplayServer::Wayland && is_hyprland_running() {
        return DesktopEnvironment::Hyprland;
    }

    DesktopEnvironment::Other(None)
}

/// Checks if Hyprland is running by trying to execute hyprctl.
fn is_hyprland_running() -> bool {
    Command::new("hyprctl")
        .arg("version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Checks if a command is available in PATH.
#[allow(dead_code)]
pub fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_server_display() {
        assert_eq!(format!("{}", DisplayServer::Wayland), "Wayland");
        assert_eq!(format!("{}", DisplayServer::X11), "X11");
        assert_eq!(format!("{}", DisplayServer::Unknown), "Unknown");
    }

    #[test]
    fn test_desktop_environment_display() {
        assert_eq!(format!("{}", DesktopEnvironment::Gnome), "GNOME");
        assert_eq!(format!("{}", DesktopEnvironment::Kde), "KDE Plasma");
        assert_eq!(format!("{}", DesktopEnvironment::Hyprland), "Hyprland");
        assert_eq!(
            format!("{}", DesktopEnvironment::Other(Some("Custom".to_string()))),
            "Custom"
        );
        assert_eq!(format!("{}", DesktopEnvironment::Other(None)), "Unknown");
    }

    #[test]
    fn test_desktop_session_detect() {
        // This test will pass regardless of the actual environment
        let session = DesktopSession::detect();
        println!("Detected session: {}", session);
        println!("Backend: {}", session.window_list_backend());
    }

    #[test]
    fn test_window_list_backend_selection() {
        let session = DesktopSession {
            display_server: DisplayServer::Wayland,
            desktop_environment: DesktopEnvironment::Hyprland,
        };
        assert_eq!(session.window_list_backend(), WindowListBackend::Hyprland);

        let session = DesktopSession {
            display_server: DisplayServer::Wayland,
            desktop_environment: DesktopEnvironment::Gnome,
        };
        assert_eq!(
            session.window_list_backend(),
            WindowListBackend::GnomeWayland
        );

        let session = DesktopSession {
            display_server: DisplayServer::X11,
            desktop_environment: DesktopEnvironment::Gnome,
        };
        assert_eq!(session.window_list_backend(), WindowListBackend::X11);
    }
}
