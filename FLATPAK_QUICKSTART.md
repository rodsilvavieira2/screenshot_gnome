# Flatpak Configuration - Quick Start Guide

## ✅ Completed Configuration

All Flatpak/Flathub configuration files have been created successfully!

### Created Files

1. **LICENSE** (1.1K) - MIT license file
2. **flathub.json** (43 bytes) - Flathub build configuration
3. **io.github.rodsilvaviera2.ScreenshotGnome.metainfo.xml** (4.2K) - AppStream metadata
4. **io.github.rodsilvaviera2.ScreenshotGnome.desktop** (873 bytes) - Desktop entry
5. **cargo-sources.json** (99K) - Cargo dependencies for offline build
6. **io.github.rodsilvaviera2.ScreenshotGnome.json** (2.3K) - Main Flatpak manifest
7. **FLATPAK.md** (12K) - Comprehensive Flatpak documentation

### Updated Files

- **README.md** - Added Flatpak installation instructions
- **.gitignore** - Added Flatpak build directories

---

## 🚀 Next Steps

### Before Testing/Submission

#### 1. Take Application Screenshots ⚠️ REQUIRED
Screenshots are currently using placeholder images. You need to:

```bash
# 1. Build and run the app
make build
./target/release/screenshot_gnome

# 2. Take screenshots (recommended: 1600x900 or 1920x1080, 16:9 ratio)
mkdir screenshots
# Save these screenshots:
# - screenshots/main-window.png - Main window with capture options
# - screenshots/editor.png - Editor interface with annotations
# - screenshots/selection.png - Selection mode in action

# 3. Commit screenshots
git add screenshots/
git commit -m "Add application screenshots"
git push

# 4. Get commit hash
git rev-parse HEAD

# 5. Update metainfo.xml with real screenshot URLs
# Replace the placeholder URLs with:
# https://raw.githubusercontent.com/rodsilvavieira2/screenshot_gnome/COMMIT_HASH/screenshots/main-window.png
```

#### 2. Create a Git Release Tag

```bash
# Ensure all changes are committed
git add .
git commit -m "Add Flatpak configuration"
git push

# Create and push tag
git tag -a v0.1.0 -m "Release v0.1.0 - Initial release with Flatpak support"
git push origin v0.1.0

# Get the commit hash for the tag
git rev-parse v0.1.0
```

#### 3. Update Flatpak Manifest with Commit Hash

Edit `io.github.rodsilvaviera2.ScreenshotGnome.json` and replace:
```json
"commit": "REPLACE_WITH_ACTUAL_COMMIT_HASH"
```

With the actual commit hash from the tag.

---

## 🧪 Testing Locally

### Install Prerequisites

```bash
# Install Flatpak (if not already installed)
sudo dnf install flatpak  # Fedora
# or
sudo apt install flatpak  # Ubuntu/Debian

# Add Flathub repository
flatpak remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo

# Install Flatpak Builder
flatpak install -y flathub org.flatpak.Builder
```

### Validate Configuration

```bash
# 1. Validate metainfo file
flatpak run --command=flatpak-builder-lint org.flatpak.Builder \
  appstream io.github.rodsilvaviera2.ScreenshotGnome.metainfo.xml

# 2. Validate manifest
flatpak run --command=flatpak-builder-lint org.flatpak.Builder \
  manifest io.github.rodsilvaviera2.ScreenshotGnome.json
```

### Build and Test

```bash
# Build and install locally
flatpak run --command=flatpak-builder org.flatpak.Builder \
  --user --install --force-clean build-dir \
  io.github.rodsilvaviera2.ScreenshotGnome.json

# Run the application
flatpak run io.github.rodsilvaviera2.ScreenshotGnome

# Test different modes
flatpak run io.github.rodsilvaviera2.ScreenshotGnome -s  # Selection
flatpak run io.github.rodsilvaviera2.ScreenshotGnome -w  # Window
flatpak run io.github.rodsilvaviera2.ScreenshotGnome --screen  # Full screen

# Validate the built repository
flatpak run --command=flatpak-builder-lint org.flatpak.Builder \
  repo build-dir/repo
```

---

## 📦 Submitting to Flathub

### Prerequisites Checklist

- [ ] Screenshots added and committed to repository
- [ ] Git tag created (e.g., v0.1.0)
- [ ] Manifest updated with actual commit hash
- [ ] Local build tested successfully
- [ ] All validations pass without errors
- [ ] Application runs correctly in Flatpak sandbox
- [ ] 2FA enabled on GitHub account

### Submission Process

#### 1. Fork Flathub Repository

Go to https://github.com/flathub/flathub and click "Fork"

**IMPORTANT:** Uncheck "Copy the master branch only" when forking!

#### 2. Clone and Prepare

```bash
# Clone with new-pr branch
git clone --branch=new-pr git@github.com:rodsilvaviera2/flathub.git
cd flathub

# Or using GitHub CLI
gh repo fork --clone flathub/flathub && cd flathub
git checkout --track origin/new-pr

# Create submission branch
git checkout -b screenshot-gnome-submission new-pr
```

#### 3. Add Files

```bash
# Copy ONLY these files (DO NOT copy source code or binaries!)
cp /path/to/screenshot_gnome/io.github.rodsilvaviera2.ScreenshotGnome.json .
cp /path/to/screenshot_gnome/cargo-sources.json .
cp /path/to/screenshot_gnome/flathub.json .

# Add to git
git add io.github.rodsilvaviera2.ScreenshotGnome.json
git add cargo-sources.json
git add flathub.json

# Commit
git commit -m "Add io.github.rodsilvaviera2.ScreenshotGnome"

# Push
git push origin screenshot-gnome-submission
```

#### 4. Create Pull Request

1. Go to https://github.com/flathub/flathub
2. Click "New Pull Request"
3. **Set base branch to: `new-pr`** (NOT master!)
4. Set compare branch to: `rodsilvaviera2:screenshot-gnome-submission`
5. Title: **"Add io.github.rodsilvaviera2.ScreenshotGnome"**
6. Fill in the PR template
7. Submit!

#### 5. During Review

- Reviewers will test your submission
- Address any feedback promptly
- You can request a test build by commenting: `bot, build`
- Do NOT close the PR to make changes - just push new commits

#### 6. After Approval

- Repository will be created at: `github.com/flathub/io.github.rodsilvaviera2.ScreenshotGnome`
- You'll receive an invitation to collaborate
- Accept within one week
- Future updates go to that repository (not flathub/flathub)

---

## 📝 Important Notes

### DO:
- ✅ Test locally before submitting
- ✅ Use actual screenshots (not placeholders)
- ✅ Use commit hashes from tags (not branch names)
- ✅ Keep manifest sources pointed at your public repository
- ✅ Respond to reviewer feedback
- ✅ Enable 2FA on GitHub

### DON'T:
- ❌ Include source code in Flathub submission
- ❌ Include compiled binaries
- ❌ Point to `main` branch in screenshot URLs
- ❌ Submit without testing locally first
- ❌ Open PR against `master` branch
- ❌ Use AI tools to generate submission content

---

## 🔧 Troubleshooting

### Common Issues

**Build fails with "Network access denied"**
- Ensure all dependencies are in cargo-sources.json
- Regenerate cargo-sources.json if you updated dependencies

**Validation errors**
- Check error messages carefully
- Most common: screenshot URLs, release dates, ID mismatches
- Use `flatpak-builder-lint` to see specific issues

**Icons not showing**
- Verify icon files exist in correct sizes
- Check icon name matches app ID
- Rebuild and reinstall

**Permission errors at runtime**
- Review finish-args in manifest
- Add necessary permissions (but keep minimal)
- Test with different permission combinations

### Getting Help

- **Documentation:** See FLATPAK.md for detailed guide
- **Flathub Docs:** https://docs.flathub.org/
- **Matrix Chat:** https://matrix.to/#/#flathub:matrix.org
- **Discourse:** https://discourse.flathub.org/
- **GitHub Issues:** https://github.com/flathub/flathub/issues

---

## 📚 Additional Resources

- [FLATPAK.md](FLATPAK.md) - Comprehensive guide
- [Flathub Requirements](https://docs.flathub.org/docs/for-app-authors/requirements)
- [Flathub Submission Process](https://docs.flathub.org/docs/for-app-authors/submission)
- [AppStream Guidelines](https://docs.flathub.org/docs/for-app-authors/metainfo-guidelines)
- [Flatpak Builder Documentation](https://docs.flatpak.org/en/latest/flatpak-builder.html)

---

## 📊 Status Summary

### Ready for Testing ✅
- [x] All configuration files created
- [x] Manifest properly structured
- [x] Permissions defined
- [x] Dependencies included
- [x] Documentation complete

### Needs Action Before Submission ⚠️
- [ ] Add real application screenshots
- [ ] Create and push git release tag
- [ ] Update manifest with commit hash
- [ ] Test local build
- [ ] Pass all validations

### After Submission ℹ️
- [ ] Monitor PR for review comments
- [ ] Accept repository invitation
- [ ] Set up update workflow

---

**Current App ID:** `io.github.rodsilvaviera2.ScreenshotGnome`  
**Repository:** https://github.com/rodsilvavieira2/screenshot_gnome  
**Target Runtime:** GNOME Platform 47

---

Good luck with your Flathub submission! 🎉
