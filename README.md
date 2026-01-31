# Screenshot Tool for GNOME

A modern screenshot utility for GNOME with built-in editing and annotation capabilities.

![Screenshot Tool](icons/256x256/screenshot_gnome.png)

## Features

- **Multiple Capture Modes**: Capture full screen, individual windows, or selected areas
- **Built-in Editor**: Annotate and edit screenshots without leaving the application
- **Annotation Tools**: 
  - Freehand drawing
  - Shapes (rectangle, ellipse, arrow)
  - Text annotations
  - Color picker for custom colors
- **Image Operations**: Crop and resize your screenshots
- **Quick Actions**: Copy to clipboard or save to file
- **Keyboard Shortcuts**: Configurable shortcuts for quick workflow
- **Modern Interface**: Built with GTK4 and libadwaita following GNOME HIG

## Requirements

### Build Dependencies

- Rust 1.70 or later
- GTK4 (>= 4.12)
- libadwaita (>= 1.5)
- pkg-config
- gcc or clang

### Runtime Dependencies

- GTK4
- libadwaita
- X11 or Wayland display server

### Installing Dependencies

**Fedora/RHEL:**
```bash
sudo dnf install rust cargo gtk4-devel libadwaita-devel
```

**Ubuntu/Debian:**
```bash
sudo apt install rustc cargo libgtk-4-dev libadwaita-1-dev build-essential
```

**Arch Linux:**
```bash
sudo pacman -S rust gtk4 libadwaita
```

## Building

Clone the repository and build the release binary:

```bash
git clone https://github.com/rodsilvavieira2/screenshot_gnome.git
cd screenshot_gnome
make build
```

Or build manually with cargo:

```bash
cargo build --release
```

The binary will be located at `target/release/screenshot_gnome`.

## Installation

### From Flathub (Recommended - Coming Soon)

Once published on Flathub, you can install with:

```bash
flatpak install flathub io.github.rodsilvaviera2.ScreenshotGnome
```

Run the application:
```bash
flatpak run io.github.rodsilvaviera2.ScreenshotGnome
```

### Building Flatpak Locally

You can build and install the Flatpak package locally:

```bash
# Install Flatpak Builder
flatpak install -y flathub org.flatpak.Builder

# Build and install
flatpak run --command=flatpak-builder org.flatpak.Builder --user --install --force-clean build-dir io.github.rodsilvaviera2.ScreenshotGnome.json

# Run the app
flatpak run io.github.rodsilvaviera2.ScreenshotGnome
```

### System-wide Installation (requires sudo)

Install to `/usr/local` (recommended):

```bash
make build
sudo make install
```

This installs:
- Binary to `/usr/local/bin/screenshot_gnome`
- Desktop file to `/usr/local/share/applications/`
- Icons to `/usr/local/share/icons/hicolor/`
- AppStream metadata to `/usr/local/share/metainfo/`

### User-local Installation (no sudo required)

Install to `~/.local` for the current user only:

```bash
make build
make install-user
```

**Note**: Make sure `~/.local/bin` is in your `PATH`. Add this to your `~/.bashrc` or `~/.zshrc`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

### Quick Installation

One-liner for system-wide installation:
```bash
make build && sudo make install
```

One-liner for user-local installation:
```bash
make build && make install-user
```

## Usage

### From Command Line

```bash
# Launch the application
screenshot_gnome

# Capture selection mode
screenshot_gnome -s
screenshot_gnome --selection

# Capture window mode
screenshot_gnome -w
screenshot_gnome --window

# Capture full screen
screenshot_gnome --screen
```

### From GNOME

1. Open **Activities** (press Super key)
2. Search for "**Screenshot Tool**"
3. Click to launch

Or right-click the icon in the menu for quick actions:
- Capture Selection
- Capture Window
- Capture Screen

### Keyboard Shortcuts

The application supports configurable keyboard shortcuts for common operations. Check the application preferences for the full list.

## Uninstallation

### System-wide Uninstallation

```bash
sudo make uninstall
```

### User-local Uninstallation

```bash
make uninstall-user
```

## Development

### Project Structure

```
screenshot_gnome/
├── src/
│   ├── app/           # Application state and configuration
│   ├── capture/       # Screen capture backends
│   ├── editor/        # Image editing and annotation tools
│   ├── ui/            # GTK4 user interface
│   └── main.rs        # Application entry point
├── icons/             # Application icons
├── Cargo.toml         # Rust dependencies
├── Makefile           # Build and installation
└── screenshot_gnome.desktop  # Desktop entry
```

### Running in Development

```bash
cargo run
```

With arguments:
```bash
cargo run -- -s    # Selection mode
cargo run -- -w    # Window mode
```

### Enabling Debug Logging

Set the `RUST_LOG` environment variable:

```bash
RUST_LOG=debug cargo run
```

or for installed binary:

```bash
RUST_LOG=debug screenshot_gnome
```

## Makefile Targets

- `make help` - Show available targets
- `make build` - Build release binary
- `make clean` - Clean build artifacts
- `sudo make install` - Install system-wide
- `sudo make uninstall` - Uninstall system-wide
- `make install-user` - Install to ~/.local
- `make uninstall-user` - Uninstall from ~/.local

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Contribution Guidelines

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Credits

Built with:
- [GTK4](https://gtk.org/) - The GTK toolkit
- [libadwaita](https://gnome.pages.gitlab.gnome.org/libadwaita/) - GNOME adaptive widgets
- [xcap](https://github.com/nashaofu/xcap) - Cross-platform screen capture
- [image-rs](https://github.com/image-rs/image) - Image processing

## Support

- **Issues**: https://github.com/rodsilvavieira2/screenshot_gnome/issues
- **Discussions**: https://github.com/rodsilvavieira2/screenshot_gnome/discussions

## Flatpak/Flathub

This application is configured for distribution via Flathub. The following files are included for Flatpak packaging:

- `io.github.rodsilvaviera2.ScreenshotGnome.json` - Flatpak manifest
- `io.github.rodsilvaviera2.ScreenshotGnome.metainfo.xml` - AppStream metadata
- `io.github.rodsilvaviera2.ScreenshotGnome.desktop` - Desktop entry
- `cargo-sources.json` - Cargo dependencies for offline build
- `flathub.json` - Flathub build configuration

### Testing Flatpak Build Locally

```bash
# Install Flatpak Builder
flatpak install -y flathub org.flatpak.Builder

# Validate the metainfo file
flatpak run --command=flatpak-builder-lint org.flatpak.Builder appstream io.github.rodsilvaviera2.ScreenshotGnome.metainfo.xml

# Validate the manifest
flatpak run --command=flatpak-builder-lint org.flatpak.Builder manifest io.github.rodsilvaviera2.ScreenshotGnome.json

# Build and install locally
flatpak run --command=flatpak-builder org.flatpak.Builder --user --install --force-clean build-dir io.github.rodsilvaviera2.ScreenshotGnome.json

# Run the app
flatpak run io.github.rodsilvaviera2.ScreenshotGnome

# Validate the built repository (after build)
flatpak run --command=flatpak-builder-lint org.flatpak.Builder repo build-dir/repo
```

### Submitting to Flathub

To submit this application to Flathub:

1. Create a git tag for the release (e.g., `v0.1.0`)
2. Update the manifest with the correct commit hash
3. Fork the [flathub/flathub](https://github.com/flathub/flathub) repository (uncheck "Copy the master branch only")
4. Clone with the `new-pr` branch: `git clone --branch=new-pr git@github.com:YOUR_USERNAME/flathub.git`
5. Create a submission branch: `git checkout -b screenshot-gnome-submission new-pr`
6. Add only these files:
   - `io.github.rodsilvaviera2.ScreenshotGnome.json`
   - `cargo-sources.json`
   - `flathub.json`
7. Commit and push
8. Open a PR against `flathub/flathub:new-pr` (NOT master!)
9. Title: "Add io.github.rodsilvaviera2.ScreenshotGnome"

For more details, see the [Flathub submission documentation](https://docs.flathub.org/docs/for-app-authors/submission).

## Roadmap

- [ ] Wayland native screenshot support
- [ ] Custom save directory preferences
- [ ] Screenshot history
- [ ] Cloud upload integration
- [ ] Video recording capabilities
- [ ] OCR text extraction from screenshots

---

Made with ❤️ for the GNOME community
