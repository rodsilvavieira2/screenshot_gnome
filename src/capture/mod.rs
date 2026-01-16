pub mod desktop;
pub mod screen;
pub mod window;
pub mod window_backends;

pub use desktop::{DesktopEnvironment, DesktopSession, DisplayServer, WindowListBackend};
pub use screen::capture_primary_monitor;
pub use window::{capture_window_by_id, list_capturable_windows, WindowCaptureError, WindowInfo};
pub use window_backends::{list_windows_for_session, list_windows_with_backend};
