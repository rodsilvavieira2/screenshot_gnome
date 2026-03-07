# AGENTS.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Common Commands

### Build

To build the project, run:

```bash
make build
```

Alternatively, you can use cargo:

```bash
cargo build --release
```

### Run

To run the application, use:

```bash
cargo run
```

To run with arguments:

```bash
cargo run -- -s # Selection mode
cargo run -- -w # Window mode
```

### Test

To run the test suite, use:

```bash
cargo test
```

### Lint

To check the code for errors and warnings, use:

```bash
cargo check
```

For a more thorough lint, use clippy:

```bash
cargo clippy
```

## Code Architecture

The project is a GTK4 application written in Rust. The code is organized into the following modules:

- `src/app`: Application state and configuration.
- `src/capture`: Screen capture backends.
- `src/editor`: Image editing and annotation tools.
- `src/ui`: GTK4 user interface.
- `src/main.rs`: Application entry point.

The application uses `libadwaita` for modern GNOME widgets and `xcap` for screen capture.
