# Current Project Status: Custom Toolbar Icons

## Goal
The objective was to improve the toolbar appearance by replacing system-dependent icons with custom SVG icons bundled within the application, ensuring a consistent look across different desktop environments, and making the toolbars solid instead of blurred.

## Changes Made So Far

1.  **Created Custom SVGs**:
    *   Created 11 custom symbolic SVG files in the `icons/scalable/actions/` directory (e.g., `app-tool-pointer-symbolic.svg`, `app-object-select-symbolic.svg`).
    *   Prefixed them with `app-` to avoid conflicts with existing system themes.
    *   The SVGs are drawn using standard basic paths with `#000000` (black) fill or stroke.

2.  **Embedded Resources via GResource**:
    *   Created a `src/resources.xml` manifest listing all 11 SVG files under the prefix `/org/example/ScreenshotGnome/icons`.
    *   Added a `build.rs` script that uses `glib-compile-resources` to compile the XML and SVGs into a binary `src/resources.gresource` bundle during the `cargo build` process.

3.  **Registered Resources in GTK**:
    *   In `src/main.rs`, added logic to statically include and register the compiled `.gresource` bundle at startup using `gtk4::gio::resources_register`.
    *   Added `app.connect_startup` logic to append the custom resource path (`/org/example/ScreenshotGnome/icons`) to the default `gtk4::IconTheme`.

4.  **Updated UI Code (`src/ui/toolbar.rs` & `src/ui/dialogs.rs`)**:
    *   Replaced all `icon_name` property references for the toolbar buttons to use the new `app-*` prefixed names.
    *   Replaced the default Libadwaita `.osd` CSS class (which causes background blur) on the `tools_box`, `crop_tools_box`, and `selection_tools_box` with a new `.custom-toolbar` class.

5.  **Added Custom CSS (`src/ui/mod.rs`)**:
    *   Injected a custom `CssProvider` at application startup to define the `.custom-toolbar` class with a solid background color (`@window_bg_color`), border, and shadow.

## The Remaining Issue: Invisible Icons in Colored Buttons

While the icons load successfully in the GTK Icon Theme (verified via a C test script), they are rendering completely invisibly when placed inside buttons that have strong background colors (specifically the "Confirm" and "Cancel" buttons on the selection/crop toolbars, which use the `.suggested-action` and `.destructive-action` CSS classes).

### Debugging Attempts:
*   **Attempt 1 (`currentColor`)**: Modified the SVGs to use `fill="currentColor"` and `stroke="currentColor"` instead of `#000000`. GTK's `librsvg` failed to parse this correctly in the context of `-symbolic` icons, and the icons remained invisible.
*   **Attempt 2 (Revert to `#000000` & Clean Rebuild)**: Reverted the SVGs back to using explicit `#000000` colors. Ran `cargo clean` and rebuilt the project to ensure no stale GResource bundles were cached. The issue persists.

### Root Cause Analysis
In GTK4, for a custom SVG to successfully participate in the symbolic recoloring pipeline (where GTK automatically changes the icon's black paths to white when rendered over a dark/colored background like a blue `.suggested-action` button):

1.  The file name *must* end in `-symbolic.svg`. (We have this).
2.  The icon *must* be loaded via the `IconTheme` mechanism, not directly as a file path. (We are doing this).
3.  **Crucially, the SVG must contain specific CSS classes on its paths.** By default, GTK looks for paths with classes like `class="warning"`, `class="error"`, or it recolors paths that have *no* explicit fill/stroke or specifically use `#000000`.

It appears that simply naming the file `-symbolic.svg` and using `#000000` is insufficient for GTK4 to automatically invert the colors inside `.suggested-action` buttons without further metadata or specific `<style>` blocks embedded in the SVG, or there is an issue with how `librsvg` is parsing the basic `<path stroke="#000000">` elements we provided (as standard GNOME symbolics heavily rely on fills rather than strokes).

## Next Steps to Fix
To resolve the invisible icons, the SVG files (specifically `app-object-select-symbolic.svg` and `app-process-stop-symbolic.svg`) need to be restructured to comply exactly with GNOME's strict symbolic icon specifications. This usually involves:
1.  Removing explicit `stroke="#000000"` and `fill="#000000"` attributes entirely.
2.  Converting stroked paths (`<path stroke="...">`) into filled composite paths (`<path fill="...">`), as `librsvg`'s symbolic recoloring heavily prioritizes `fill` over `stroke` when applying the GTK theme's foreground color.
3.  Adding a `<style>` block if necessary to define the standard GNOME symbolic CSS context.
