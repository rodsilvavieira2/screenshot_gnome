# Project Overview
`screenshot_gnome` is a modern screenshot utility for GNOME built with Rust, GTK4, and libadwaita. It features multiple capture modes (full screen, individual windows, selected areas), a built-in editor with annotation tools, image operations, and configurable keyboard shortcuts.

# Architecture & Technologies
- **Language:** Rust (Edition 2021)
- **UI Framework:** GTK4 (>= 4.12) and libadwaita (>= 1.5)
- **Key Dependencies:**
  - `xcap` (v0.8.0) for cross-platform screen capture.
  - `image` (v0.25.9) for image processing.
  - `log` and `env_logger` for logging.

# Building and Running

## Build
```bash
# Build the release binary using Cargo
cargo build --release

# Alternatively, using the Makefile
make build
```

## Run
```bash
# Run in development mode
cargo run

# Run with specific arguments
cargo run -- -s       # Capture selection mode
cargo run -- -w       # Capture window mode
cargo run -- --screen # Capture full screen
```

## Installation
The project provides several installation options via the `Makefile`:
- `make build && sudo make install` - System-wide installation (to `/usr/local`)
- `make build && make install-user` - User-local installation (to `~/.local`)
- `sudo make uninstall` - System-wide uninstallation
- `make uninstall-user` - User-local uninstallation

## Flatpak Support
The application is configured for Flatpak distribution. You can build and install it locally using:
```bash
# Ensure Flatpak Builder is installed
flatpak install -y flathub org.flatpak.Builder

# Build and install locally
flatpak run --command=flatpak-builder org.flatpak.Builder --user --install --force-clean build-dir io.github.rodsilvaviera2.ScreenshotGnome.json

# Run the app
flatpak run io.github.rodsilvaviera2.ScreenshotGnome
```

# Development Conventions
- **Logging:** Enable debug logging by setting the `RUST_LOG` environment variable to `debug` (e.g., `RUST_LOG=debug cargo run`).
- **Project Structure:**
  - `src/app/`: Application state and configuration.
  - `src/capture/`: Screen capture backends.
  - `src/editor/`: Image editing and annotation tools.
  - `src/ui/`: GTK4 user interface components.
  - `src/main.rs`: Application entry point.
- **Contributions:** Create feature branches (e.g., `feature/amazing-feature`) and submit Pull Requests on GitHub.
