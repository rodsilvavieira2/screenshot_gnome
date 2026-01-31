# Flatpak Configuration Guide

This document explains the Flatpak/Flathub configuration for Screenshot Tool.

## Overview

Screenshot Tool is configured for distribution via Flathub using the GNOME Platform 47 runtime. The application uses Rust and requires GTK4 and libadwaita.

## Application ID

**App ID:** `io.github.rodsilvaviera2.ScreenshotGnome`

This follows the Flathub requirements for code hosting IDs:
- Format: `io.github.[username].[AppName]`
- Repository: https://github.com/rodsilvavieira2/screenshot_gnome

## Files Overview

### Core Flatpak Files

1. **`io.github.rodsilvaviera2.ScreenshotGnome.json`**
   - Main Flatpak manifest
   - Defines build process, dependencies, and permissions
   - Uses GNOME Platform 47 runtime
   - Includes Rust SDK extension

2. **`cargo-sources.json`**
   - Generated file containing all Cargo dependencies
   - Required because Flatpak builds have no network access
   - Contains URLs and checksums for all crates from crates.io
   - Regenerate when `Cargo.lock` changes

3. **`flathub.json`**
   - Flathub-specific build configuration
   - Specifies architectures: x86_64 and aarch64

### Metadata Files

4. **`io.github.rodsilvaviera2.ScreenshotGnome.metainfo.xml`**
   - AppStream metadata (required by Flathub)
   - Contains app description, screenshots, releases, etc.
   - Must pass validation with `appstreamcli`

5. **`io.github.rodsilvaviera2.ScreenshotGnome.desktop`**
   - Desktop entry file
   - Defines application name, icon, categories
   - Includes quick actions for different capture modes

6. **`LICENSE`**
   - MIT license file
   - Required by Flathub
   - Must match `project_license` in metainfo

## Permissions (finish-args)

The application requests the following permissions:

### Display Access
- `--share=ipc` - Required for X11
- `--socket=fallback-x11` - X11 display access (for screenshots on X11)
- `--socket=wayland` - Wayland display access
- `--device=dri` - GPU acceleration for rendering

### File System Access
- `--filesystem=xdg-pictures/Screenshots:create` - Create Screenshots folder in Pictures
- `--filesystem=xdg-download:rw` - Read/write access to Downloads folder

### D-Bus Services
- `--talk-name=org.freedesktop.portal.Screenshot` - XDG Screenshot portal
- `--talk-name=org.gnome.Shell.Screenshot` - GNOME Shell screenshot API

## Building Locally

### Prerequisites

```bash
# Install Flatpak and Flathub
sudo dnf install flatpak  # or apt/pacman depending on distro
flatpak remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo

# Install Flatpak Builder
flatpak install -y flathub org.flatpak.Builder
```

### Build Steps

```bash
# 1. Validate metainfo
flatpak run --command=flatpak-builder-lint org.flatpak.Builder \
  appstream io.github.rodsilvaviera2.ScreenshotGnome.metainfo.xml

# 2. Validate manifest
flatpak run --command=flatpak-builder-lint org.flatpak.Builder \
  manifest io.github.rodsilvaviera2.ScreenshotGnome.json

# 3. Build and install
flatpak run --command=flatpak-builder org.flatpak.Builder \
  --user --install --force-clean build-dir \
  io.github.rodsilvaviera2.ScreenshotGnome.json

# 4. Run the application
flatpak run io.github.rodsilvaviera2.ScreenshotGnome

# 5. Validate the built repository
flatpak run --command=flatpak-builder-lint org.flatpak.Builder \
  repo build-dir/repo
```

### Quick Build Command

```bash
flatpak run --command=flathub-build org.flatpak.Builder \
  --install io.github.rodsilvaviera2.ScreenshotGnome.json
```

## Regenerating cargo-sources.json

When you update dependencies in `Cargo.toml` or `Cargo.lock`:

```bash
# Option 1: Using flatpak-cargo-generator (recommended)
git clone https://github.com/flatpak/flatpak-builder-tools.git
cd flatpak-builder-tools/cargo
python3 flatpak-cargo-generator.py /path/to/Cargo.lock -o cargo-sources.json

# Option 2: Using the included simple generator
python3 << 'EOF'
import json
import re

def parse_cargo_lock(filename):
    with open(filename, 'r') as f:
        content = f.read()
    packages = []
    blocks = content.split('[[package]]')[1:]
    for block in blocks:
        lines = [l.strip() for l in block.strip().split('\n') if l.strip()]
        pkg = {}
        for line in lines:
            if line.startswith('name ='):
                pkg['name'] = line.split('"')[1]
            elif line.startswith('version ='):
                pkg['version'] = line.split('"')[1]
            elif line.startswith('source =') and 'registry' in line:
                pkg['source'] = 'registry'
            elif line.startswith('checksum ='):
                pkg['checksum'] = line.split('"')[1]
        if pkg.get('source') == 'registry' and 'checksum' in pkg:
            packages.append(pkg)
    return packages

packages = parse_cargo_lock('Cargo.lock')
sources = []
for pkg in packages:
    sources.append({
        "type": "file",
        "url": f"https://static.crates.io/crates/{pkg['name']}/{pkg['name']}-{pkg['version']}.crate",
        "sha256": pkg['checksum'],
        "dest": "cargo/vendor",
        "dest-filename": f"{pkg['name']}-{pkg['version']}.crate"
    })

sources.append({
    "type": "inline",
    "dest": "cargo",
    "dest-filename": "config.toml",
    "contents": "[source.crates-io]\nreplace-with = \"vendored-sources\"\n\n[source.vendored-sources]\ndirectory = \"vendor\"\n"
})

with open('cargo-sources.json', 'w') as f:
    json.dump(sources, f, indent=2)
EOF
```

## Updating Screenshots

Currently using placeholder screenshots. To add real screenshots:

1. **Take Screenshots:**
   - Main window showing capture options
   - Editor interface with annotation tools
   - Selection mode in action
   - Recommended resolution: 1600x900 or 1920x1080 (16:9 aspect ratio)

2. **Add to Repository:**
   ```bash
   mkdir screenshots
   # Add your PNG files
   git add screenshots/
   git commit -m "Add application screenshots"
   ```

3. **Update metainfo.xml:**
   ```xml
   <screenshot type="default">
     <image>https://raw.githubusercontent.com/rodsilvavieira2/screenshot_gnome/COMMIT_HASH/screenshots/main-window.png</image>
     <caption>Main application window</caption>
   </screenshot>
   ```
   
   Replace `COMMIT_HASH` with actual commit hash (don't use `main` branch).

4. **Update Flathub submission** with new metainfo.xml

## Flathub Submission Process

### Before Submission

1. **Create a Release Tag:**
   ```bash
   git tag -a v0.1.0 -m "Release v0.1.0"
   git push origin v0.1.0
   ```

2. **Get Commit Hash:**
   ```bash
   git rev-parse v0.1.0
   ```

3. **Update Manifest:**
   Edit `io.github.rodsilvaviera2.ScreenshotGnome.json`:
   ```json
   "sources": [
     {
       "type": "git",
       "url": "https://github.com/rodsilvavieira2/screenshot_gnome.git",
       "tag": "v0.1.0",
       "commit": "ACTUAL_COMMIT_HASH_HERE"
     },
     "cargo-sources.json"
   ]
   ```

4. **Test Build Locally:**
   ```bash
   flatpak run --command=flathub-build org.flatpak.Builder \
     --install io.github.rodsilvaviera2.ScreenshotGnome.json
   ```

### Submission Steps

1. **Fork Flathub Repository:**
   - Go to https://github.com/flathub/flathub
   - Click "Fork"
   - **IMPORTANT:** Uncheck "Copy the master branch only"

2. **Clone Your Fork:**
   ```bash
   git clone --branch=new-pr git@github.com:rodsilvaviera2/flathub.git
   cd flathub
   ```
   
   Or using GitHub CLI:
   ```bash
   gh repo fork --clone flathub/flathub && cd flathub
   git checkout --track origin/new-pr
   ```

3. **Create Submission Branch:**
   ```bash
   git checkout -b screenshot-gnome-submission new-pr
   ```

4. **Add Files:**
   ```bash
   # Copy files from your project
   cp /path/to/screenshot_gnome/io.github.rodsilvaviera2.ScreenshotGnome.json .
   cp /path/to/screenshot_gnome/cargo-sources.json .
   cp /path/to/screenshot_gnome/flathub.json .
   
   # Add to git
   git add io.github.rodsilvaviera2.ScreenshotGnome.json
   git add cargo-sources.json
   git add flathub.json
   ```

5. **Commit and Push:**
   ```bash
   git commit -m "Add io.github.rodsilvaviera2.ScreenshotGnome"
   git push origin screenshot-gnome-submission
   ```

6. **Open Pull Request:**
   - Go to https://github.com/flathub/flathub
   - Click "New Pull Request"
   - **IMPORTANT:** Set base branch to `new-pr` (NOT master!)
   - Set compare branch to your `screenshot-gnome-submission` branch
   - Title: "Add io.github.rodsilvaviera2.ScreenshotGnome"
   - Fill in the PR template

7. **Wait for Review:**
   - Reviewers will test and review your submission
   - Address any feedback
   - You can request a test build by commenting: `bot, build`

### After Approval

1. **Accept Repository Invitation:**
   - After merge, you'll get an invitation to the new repository
   - Repository will be at: https://github.com/flathub/io.github.rodsilvaviera2.ScreenshotGnome
   - Enable 2FA on GitHub if not already enabled
   - Accept invitation within one week

2. **Future Updates:**
   - Updates go directly to the new repository (no PR to flathub/flathub needed)
   - See [Flathub Maintenance Guide](https://docs.flathub.org/docs/for-app-authors/maintenance)

## Common Issues and Solutions

### Build Fails: Missing Cargo Dependencies

**Problem:** Build fails with missing crate errors

**Solution:** Regenerate `cargo-sources.json`:
```bash
# Ensure Cargo.lock is up to date
cargo update
# Regenerate sources
python3 /path/to/flatpak-cargo-generator.py Cargo.lock -o cargo-sources.json
```

### Validation Errors

**Problem:** `flatpak-builder-lint` reports errors

**Solution:** 
- Check the error message carefully
- Common issues:
  - Screenshots: Must use HTTPS URLs from tagged commits
  - Release dates: Must not be in the future
  - ID mismatch: Ensure all files use same app ID

### Permission Denied

**Problem:** App can't save screenshots

**Solution:** Check finish-args in manifest. May need to add:
```json
"--filesystem=home"  // Grants full home directory access (use sparingly)
```

### Icons Not Showing

**Problem:** App icon doesn't appear in launcher

**Solution:** 
- Verify icon files are installed to correct location
- Check icon name matches app ID
- Run `gtk-update-icon-cache` (done automatically by Flatpak)

## Resources

- [Flathub Documentation](https://docs.flathub.org/)
- [Flatpak Documentation](https://docs.flatpak.org/)
- [Flatpak Builder Tools](https://github.com/flatpak/flatpak-builder-tools)
- [AppStream Documentation](https://www.freedesktop.org/software/appstream/docs/)
- [GNOME Platform](https://docs.flatpak.org/en/latest/available-runtimes.html#gnome)

## Verification

After publication, you can request verification:
- See [Flathub Verification Guide](https://docs.flathub.org/docs/for-app-authors/verification)
- Requires control over domain or GitHub repository
- Shows verified badge on Flathub

## Maintenance

### Updating the Flatpak

1. Make changes to source code
2. Update version in `Cargo.toml` and metainfo.xml
3. Add release notes to metainfo.xml
4. Create git tag: `git tag v0.2.0`
5. Update manifest with new tag/commit
6. Regenerate cargo-sources.json if dependencies changed
7. Test build locally
8. Push to Flathub repository

### Monitoring

- Check build status: https://buildbot.flathub.org/
- View app page: https://flathub.org/apps/io.github.rodsilvaviera2.ScreenshotGnome
- Monitor issues in Flathub repository

---

For questions or help, contact:
- Email: rodsilvavieira@gmail.com
- Issues: https://github.com/rodsilvavieira2/screenshot_gnome/issues
- Flathub Matrix: https://matrix.to/#/#flathub:matrix.org
