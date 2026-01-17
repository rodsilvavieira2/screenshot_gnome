use std::env;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayServer {
    Wayland,
    X11,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesktopEnvironment {
    Gnome,
    Kde,
    Hyprland,
    Sway,
    Cinnamon,
    Xfce,
    Mate,
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

#[derive(Debug, Clone)]
pub struct DesktopSession {
    pub display_server: DisplayServer,
    pub desktop_environment: DesktopEnvironment,
}

impl DesktopSession {
    pub fn detect() -> Self {
        let display_server = detect_display_server();
        let desktop_environment = detect_desktop_environment(&display_server);

        Self {
            display_server,
            desktop_environment,
        }
    }

    #[allow(dead_code)]
    pub fn is_wayland(&self) -> bool {
        self.display_server == DisplayServer::Wayland
    }

    #[allow(dead_code)]
    pub fn is_x11(&self) -> bool {
        self.display_server == DisplayServer::X11
    }

    #[allow(dead_code)]
    pub fn is_gnome(&self) -> bool {
        self.desktop_environment == DesktopEnvironment::Gnome
    }

    #[allow(dead_code)]
    pub fn is_kde(&self) -> bool {
        self.desktop_environment == DesktopEnvironment::Kde
    }

    #[allow(dead_code)]
    pub fn is_hyprland(&self) -> bool {
        self.desktop_environment == DesktopEnvironment::Hyprland
    }

    #[allow(dead_code)]
    pub fn is_sway(&self) -> bool {
        self.desktop_environment == DesktopEnvironment::Sway
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowListBackend {
    Hyprland,
    Sway,
    GnomeWayland,
    KdeWayland,
    X11,
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

fn detect_display_server() -> DisplayServer {
    if let Ok(session_type) = env::var("XDG_SESSION_TYPE") {
        match session_type.to_lowercase().as_str() {
            "wayland" => return DisplayServer::Wayland,
            "x11" => return DisplayServer::X11,
            _ => {}
        }
    }

    if env::var("WAYLAND_DISPLAY").is_ok() {
        return DisplayServer::Wayland;
    }

    if env::var("DISPLAY").is_ok() {
        return DisplayServer::X11;
    }

    DisplayServer::Unknown
}

fn detect_desktop_environment(display_server: &DisplayServer) -> DesktopEnvironment {
    if env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
        return DesktopEnvironment::Hyprland;
    }

    if env::var("SWAYSOCK").is_ok() {
        return DesktopEnvironment::Sway;
    }

    if let Ok(current_desktop) = env::var("XDG_CURRENT_DESKTOP") {
        let desktop_lower = current_desktop.to_lowercase();

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

        if !current_desktop.is_empty() {
            return DesktopEnvironment::Other(Some(current_desktop));
        }
    }

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

    if env::var("KDE_FULL_SESSION").is_ok() {
        return DesktopEnvironment::Kde;
    }

    if env::var("GNOME_DESKTOP_SESSION_ID").is_ok() {
        return DesktopEnvironment::Gnome;
    }

    if *display_server == DisplayServer::Wayland && is_hyprland_running() {
        return DesktopEnvironment::Hyprland;
    }

    DesktopEnvironment::Other(None)
}

fn is_hyprland_running() -> bool {
    Command::new("hyprctl")
        .arg("version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
