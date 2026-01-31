# Makefile for screenshot_gnome
# A GNOME screenshot tool with editing capabilities

# Configuration
BINARY_NAME = screenshot_gnome
DESKTOP_FILE = screenshot_gnome.desktop
APPDATA_FILE = screenshot_gnome.appdata.xml
ICON_NAME = screenshot_gnome

# Installation paths (system-wide)
PREFIX ?= /usr/local
BINDIR = $(PREFIX)/bin
DATADIR = $(PREFIX)/share
APPLICATIONSDIR = $(DATADIR)/applications
METAINFODIR = $(DATADIR)/metainfo
ICONDIR = $(DATADIR)/icons/hicolor

# User-local installation paths
USER_PREFIX = $(HOME)/.local
USER_BINDIR = $(USER_PREFIX)/bin
USER_DATADIR = $(USER_PREFIX)/share
USER_APPLICATIONSDIR = $(USER_DATADIR)/applications
USER_METAINFODIR = $(USER_DATADIR)/metainfo
USER_ICONDIR = $(USER_DATADIR)/icons/hicolor

# Build configuration
CARGO = cargo
CARGO_BUILD_FLAGS = --release
TARGET_DIR = target/release
BINARY_PATH = $(TARGET_DIR)/$(BINARY_NAME)

# Icon sizes
ICON_SIZES = 48x48 64x64 128x128 256x256

# Colors for output
COLOR_RESET = \033[0m
COLOR_BOLD = \033[1m
COLOR_GREEN = \033[32m
COLOR_YELLOW = \033[33m
COLOR_BLUE = \033[34m

.PHONY: all build clean install uninstall install-user uninstall-user help

# Default target
all: build

help:
	@echo "$(COLOR_BOLD)screenshot_gnome - Installation targets$(COLOR_RESET)"
	@echo ""
	@echo "$(COLOR_GREEN)Build targets:$(COLOR_RESET)"
	@echo "  make build          - Build the release binary"
	@echo "  make clean          - Clean build artifacts"
	@echo ""
	@echo "$(COLOR_GREEN)System-wide installation (requires sudo):$(COLOR_RESET)"
	@echo "  sudo make install   - Install to $(PREFIX)"
	@echo "  sudo make uninstall - Uninstall from $(PREFIX)"
	@echo ""
	@echo "$(COLOR_GREEN)User-local installation (no sudo required):$(COLOR_RESET)"
	@echo "  make install-user   - Install to ~/.local"
	@echo "  make uninstall-user - Uninstall from ~/.local"
	@echo ""
	@echo "$(COLOR_GREEN)Quick installation:$(COLOR_RESET)"
	@echo "  make build && sudo make install"
	@echo "  make build && make install-user"

# Build the release binary
build:
	@echo "$(COLOR_BLUE)Building $(BINARY_NAME)...$(COLOR_RESET)"
	$(CARGO) build $(CARGO_BUILD_FLAGS)
	@echo "$(COLOR_GREEN)✓ Build complete: $(BINARY_PATH)$(COLOR_RESET)"

# Clean build artifacts
clean:
	@echo "$(COLOR_YELLOW)Cleaning build artifacts...$(COLOR_RESET)"
	$(CARGO) clean
	@echo "$(COLOR_GREEN)✓ Clean complete$(COLOR_RESET)"

# System-wide installation
install: check-binary
	@echo "$(COLOR_BLUE)Installing $(BINARY_NAME) to $(PREFIX)...$(COLOR_RESET)"
	
	# Install binary
	@echo "Installing binary to $(BINDIR)..."
	install -Dm755 $(BINARY_PATH) $(BINDIR)/$(BINARY_NAME)
	
	# Install desktop file
	@echo "Installing desktop file..."
	install -Dm644 $(DESKTOP_FILE) $(APPLICATIONSDIR)/$(DESKTOP_FILE)
	
	# Install AppStream metadata
	@echo "Installing AppStream metadata..."
	install -Dm644 $(APPDATA_FILE) $(METAINFODIR)/$(APPDATA_FILE)
	
	# Install icons
	@echo "Installing icons..."
	@for size in $(ICON_SIZES); do \
		if [ -f "icons/$${size}/$(ICON_NAME).png" ]; then \
			install -Dm644 "icons/$${size}/$(ICON_NAME).png" \
				"$(ICONDIR)/$${size}/apps/$(ICON_NAME).png"; \
		fi; \
	done
	
	# Update icon cache
	@echo "Updating icon cache..."
	@if command -v gtk-update-icon-cache >/dev/null 2>&1; then \
		gtk-update-icon-cache -q -t -f $(ICONDIR) 2>/dev/null || true; \
	fi
	
	# Update desktop database
	@echo "Updating desktop database..."
	@if command -v update-desktop-database >/dev/null 2>&1; then \
		update-desktop-database -q $(APPLICATIONSDIR) 2>/dev/null || true; \
	fi
	
	@echo ""
	@echo "$(COLOR_GREEN)✓ Installation complete!$(COLOR_RESET)"
	@echo ""
	@echo "$(COLOR_BOLD)You can now run:$(COLOR_RESET)"
	@echo "  $(BINARY_NAME)              - Launch the application"
	@echo "  $(BINARY_NAME) -s           - Capture selection"
	@echo "  $(BINARY_NAME) -w           - Capture window"
	@echo "  $(BINARY_NAME) --screen     - Capture screen"
	@echo ""
	@echo "$(COLOR_BOLD)Or launch from GNOME Activities/Menu$(COLOR_RESET)"

# System-wide uninstallation
uninstall:
	@echo "$(COLOR_YELLOW)Uninstalling $(BINARY_NAME) from $(PREFIX)...$(COLOR_RESET)"
	
	# Remove binary
	rm -f $(BINDIR)/$(BINARY_NAME)
	
	# Remove desktop file
	rm -f $(APPLICATIONSDIR)/$(DESKTOP_FILE)
	
	# Remove AppStream metadata
	rm -f $(METAINFODIR)/$(APPDATA_FILE)
	
	# Remove icons
	@for size in $(ICON_SIZES); do \
		rm -f "$(ICONDIR)/$${size}/apps/$(ICON_NAME).png"; \
	done
	
	# Update icon cache
	@if command -v gtk-update-icon-cache >/dev/null 2>&1; then \
		gtk-update-icon-cache -q -t -f $(ICONDIR) 2>/dev/null || true; \
	fi
	
	# Update desktop database
	@if command -v update-desktop-database >/dev/null 2>&1; then \
		update-desktop-database -q $(APPLICATIONSDIR) 2>/dev/null || true; \
	fi
	
	@echo "$(COLOR_GREEN)✓ Uninstallation complete$(COLOR_RESET)"

# User-local installation
install-user: check-binary
	@echo "$(COLOR_BLUE)Installing $(BINARY_NAME) to ~/.local...$(COLOR_RESET)"
	
	# Create directories
	mkdir -p $(USER_BINDIR)
	mkdir -p $(USER_APPLICATIONSDIR)
	mkdir -p $(USER_METAINFODIR)
	
	# Install binary
	@echo "Installing binary to $(USER_BINDIR)..."
	install -Dm755 $(BINARY_PATH) $(USER_BINDIR)/$(BINARY_NAME)
	
	# Install desktop file
	@echo "Installing desktop file..."
	install -Dm644 $(DESKTOP_FILE) $(USER_APPLICATIONSDIR)/$(DESKTOP_FILE)
	
	# Install AppStream metadata
	@echo "Installing AppStream metadata..."
	install -Dm644 $(APPDATA_FILE) $(USER_METAINFODIR)/$(APPDATA_FILE)
	
	# Install icons
	@echo "Installing icons..."
	@for size in $(ICON_SIZES); do \
		if [ -f "icons/$${size}/$(ICON_NAME).png" ]; then \
			mkdir -p "$(USER_ICONDIR)/$${size}/apps"; \
			install -Dm644 "icons/$${size}/$(ICON_NAME).png" \
				"$(USER_ICONDIR)/$${size}/apps/$(ICON_NAME).png"; \
		fi; \
	done
	
	# Update icon cache
	@echo "Updating icon cache..."
	@if command -v gtk-update-icon-cache >/dev/null 2>&1; then \
		gtk-update-icon-cache -q -t -f $(USER_ICONDIR) 2>/dev/null || true; \
	fi
	
	# Update desktop database
	@echo "Updating desktop database..."
	@if command -v update-desktop-database >/dev/null 2>&1; then \
		update-desktop-database -q $(USER_APPLICATIONSDIR) 2>/dev/null || true; \
	fi
	
	@echo ""
	@echo "$(COLOR_GREEN)✓ Installation complete!$(COLOR_RESET)"
	@echo ""
	@echo "$(COLOR_BOLD)Make sure ~/.local/bin is in your PATH$(COLOR_RESET)"
	@echo "You can now run:"
	@echo "  $(BINARY_NAME)              - Launch the application"
	@echo "  $(BINARY_NAME) -s           - Capture selection"
	@echo "  $(BINARY_NAME) -w           - Capture window"
	@echo "  $(BINARY_NAME) --screen     - Capture screen"
	@echo ""
	@echo "$(COLOR_BOLD)Or launch from GNOME Activities/Menu$(COLOR_RESET)"

# User-local uninstallation
uninstall-user:
	@echo "$(COLOR_YELLOW)Uninstalling $(BINARY_NAME) from ~/.local...$(COLOR_RESET)"
	
	# Remove binary
	rm -f $(USER_BINDIR)/$(BINARY_NAME)
	
	# Remove desktop file
	rm -f $(USER_APPLICATIONSDIR)/$(DESKTOP_FILE)
	
	# Remove AppStream metadata
	rm -f $(USER_METAINFODIR)/$(APPDATA_FILE)
	
	# Remove icons
	@for size in $(ICON_SIZES); do \
		rm -f "$(USER_ICONDIR)/$${size}/apps/$(ICON_NAME).png"; \
	done
	
	# Update icon cache
	@if command -v gtk-update-icon-cache >/dev/null 2>&1; then \
		gtk-update-icon-cache -q -t -f $(USER_ICONDIR) 2>/dev/null || true; \
	fi
	
	# Update desktop database
	@if command -v update-desktop-database >/dev/null 2>&1; then \
		update-desktop-database -q $(USER_APPLICATIONSDIR) 2>/dev/null || true; \
	fi
	
	@echo "$(COLOR_GREEN)✓ Uninstallation complete$(COLOR_RESET)"

# Check if binary exists
check-binary:
	@if [ ! -f "$(BINARY_PATH)" ]; then \
		echo "$(COLOR_YELLOW)Error: Binary not found at $(BINARY_PATH)$(COLOR_RESET)"; \
		echo "Please run 'make build' first"; \
		exit 1; \
	fi
