# Screenshot GNOME - AI Coding Instructions

This project is a Rust-based screenshot and image annotation tool for GNOME, built using GTK4, `libadwaita`, and `xcap`.

## Architecture Overview
- **State Management**: Uses a centralized `AppState` (in `src/app/state.rs`) wrapped in `Rc<RefCell<AppState>>` for shared access across UI components.
- **UI Structure**: Organized into modular components (header, toolbar, drawing area) initialized in `src/ui/mod.rs`. Components use a "Components" struct pattern (e.g., `DrawingComponents`).
- **Capture Logic**: Uses `xcap` for cross-platform screen and window capture, encapsulated in `src/capture/`.
- **Desktop Detection**: The `src/capture/desktop.rs` module detects the current desktop environment and display server to use the appropriate window listing backend.
- **Annotation System**: Implemented using Cairo drawing on a `GtkDrawingArea`. Annotations are stored as a list of enums (`Rectangle`, `FreeDraw`, `Text`) in `src/editor/annotations.rs`.
- **Coordinate Systems**: The `EditorState` manages transformations between `display_coords` (UI/Cairo space) and `image_coords` (original pixel space). Always use `display_to_image_coords` when handling mouse input.

## Desktop Environment Detection

The application automatically detects the desktop session to use the appropriate method for listing windows:

### Supported Environments
| Desktop Environment | Display Server | Backend Used |
|---------------------|----------------|--------------|
| Hyprland | Wayland | `hyprctl clients -j` |
| Sway | Wayland | `swaymsg -t get_tree` |
| GNOME | Wayland | D-Bus (`org.gnome.Shell.Introspect`) |
| KDE Plasma | Wayland | D-Bus/kdotool |
| Any | X11 | xcap library |

### Key Types
- `DesktopSession`: Combined detection result with `display_server` and `desktop_environment`.
- `DisplayServer`: Enum with `Wayland`, `X11`, `Unknown` variants.
- `DesktopEnvironment`: Enum with `Gnome`, `Kde`, `Hyprland`, `Sway`, `Cinnamon`, `Xfce`, `Mate`, `Other(Option<String>)`.
- `WindowListBackend`: The recommended backend for listing windows.

### Usage
```rust
use crate::capture::desktop::DesktopSession;

let session = DesktopSession::detect();
println!("Running on: {}", session); // e.g., "GNOME on Wayland"
println!("Backend: {}", session.window_list_backend()); // e.g., "GNOME Wayland (D-Bus)"

// Use the smart window listing (auto-selects backend)
let windows = list_capturable_windows()?;
```

## Key Developer Workflows
- **Build**: `cargo build`
- **Run**: `cargo run` (requires a Wayland or X11 session for GTK/xcap)
- **Adding a Tool**:
  1. Add variant to `EditorTool` in `src/editor/tools.rs`.
  2. Update `on_drag_start/update/end` in `src/editor/mod.rs`.
  3. Implement drawing logic in `Annotation::draw` in `src/editor/annotations.rs`.
- **Adding a New Desktop Backend**:
  1. Add variant to `DesktopEnvironment` in `src/capture/desktop.rs`.
  2. Add detection logic in `detect_desktop_environment()`.
  3. Add backend variant to `WindowListBackend` and update `window_list_backend()`.
  4. Implement the backend in `src/capture/window_backends.rs`.

## Coding Conventions
- **GTK Patterns**: Prefer the builder pattern for GTK widgets (e.g., `gtk::Box::builder().build()`).
- **Modularity**: UI event handlers are separated from widget creation. See `src/ui/handlers.rs` for event connection logic.
- **Error Handling**: Use `Result<T, String>` for capture and clipboard operations to provide user-facing error messages. Use `WindowCaptureError` for window-related operations.
- **Cairo Drawing**: Use `cr.save()` and `cr.restore()` when performing transformations inside `draw_content`.
- **JSON Parsing**: The window backends use manual JSON parsing to avoid external dependencies. Consider using `serde_json` if complexity increases.

## Important Files
- `src/app/state.rs`: The "Source of Truth" for application state.
- `src/ui/handlers.rs`: Main entry point for all UI interactions and business logic integration.
- `src/ui/drawing.rs`: Contains the Cairo rendering pipeline for the image and annotations.
- `src/ui/dialogs.rs`: Window selector dialog that displays detected session info.
- `src/editor/mod.rs`: Coordinates annotation logic and coordinate transforms.
- `src/capture/desktop.rs`: Desktop environment and display server detection.
- `src/capture/window.rs`: Window listing and capture using smart backend selection.
- `src/capture/window_backends.rs`: Backend implementations for different desktop environments.