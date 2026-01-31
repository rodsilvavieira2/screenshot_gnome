# Screenshot GNOME - AI Coding Instructions

Rust-based screenshot and annotation tool for GNOME (GTK4/Libadwaita).

## Core Architecture
- **State**: Centralized in `AppState` (`src/app/state.rs`), shared via `Rc<RefCell<AppState>>`.
- **UI Logic**: Separated into component creation (`src/ui/*.rs`) and event connection (`src/ui/handlers.rs`). Use the `UiComponents` struct to pass widgets around.
- **Coordinates**: Transformations between `display_coords` (Cairo/UI) and `image_coords` (Pixbuf) are managed in `EditorState` (`src/editor/mod.rs`). Always use `display_to_image_coords` for mouse input.
- **Capture**: Smart backend selection based on `DesktopSession` (`src/capture/desktop.rs`). Supports Wayland (grim, gnome-screenshot, spectacle) and X11 (xcap).

## Project Patterns & Conventions
- **GTK Widgets**: Use the builder pattern: `gtk::Box::builder().orientation(Orientation::Vertical).build()`.
- **Drawing**: Cairo operations happen in `src/ui/drawing.rs` within `draw_content`. Use `cr.save()` and `cr.restore()` for transforms.
- **Annotations**: Implemented in `src/editor/annotations.rs`. New shapes must implement `hit_test`, `move_by`, and `draw`.
- **Backends**: Window listing backends in `src/capture/window_backends.rs` often use manual JSON/GVariant parsing to minimize dependencies.
- **Logging**: Use `log` crate (`debug!`, `info!`, `error!`) instead of `println!`.

## Critical Files
- `src/app/state.rs`: The "Source of Truth".
- `src/ui/handlers.rs`: Main hub for UI interaction logic and signal connections.
- `src/editor/mod.rs`: Manages annotation tools and coordinate scaling.
- `src/capture/window_backends.rs`: OS-specific window detection and capture logic.

## Workflow Commands
- **Build/Run**: `cargo run` (Requires Wayland/X11 session).
- **CLI Options**: Supports `--selection`, `--screen`, and `--window` flags.
- **Tests**: `cargo test` (Includes backend detection tests).

## Example: Adding a UI Event
```rust
// In src/ui/handlers.rs
pub fn connect_custom_handler(state: &Rc<RefCell<AppState>>, components: &UiComponents) {
    components.toolbar.custom_btn.connect_clicked({
        let state = state.clone();
        move |_| {
            let mut s = state.borrow_mut();
            // modify state or trigger action
        }
    });
}
```
