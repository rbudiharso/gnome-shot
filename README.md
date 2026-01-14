# GNOME Shot

A screenshot and annotation tool for GNOME, inspired by Shottr on macOS.

## Features

- Screenshot capture via XDG Desktop Portal (GNOME/Wayland compatible)
- Annotation tools: Arrow, Rectangle, Line, Ellipse, Highlight, Blur
- Color picker with preset colors
- Undo/Redo support
- Save to file and copy to clipboard
- Quick save & exit with Escape key
- GNOME Shell extension for global shortcut

## Requirements

- GNOME 45+ (Wayland or X11)
- Rust 1.70+
- GTK4 and libadwaita development libraries
- wl-clipboard (for Wayland clipboard support)

### Fedora

```bash
sudo dnf install gtk4-devel libadwaita-devel glib2-devel cairo-devel \
    pango-devel graphene-devel gdk-pixbuf2-devel wl-clipboard
```

### Ubuntu/Debian

```bash
sudo apt install libgtk-4-dev libadwaita-1-dev libglib2.0-dev libcairo2-dev \
    libpango1.0-dev libgraphene-1.0-dev libgdk-pixbuf-2.0-dev wl-clipboard
```

### Arch Linux

```bash
sudo pacman -S gtk4 libadwaita glib2 cairo pango graphene gdk-pixbuf2 wl-clipboard
```

## Installation

### Flatpak (recommended)

```bash
# Install from Flathub (coming soon)
flatpak install flathub org.gnome.GnomeShot

# Or build locally
flatpak install flathub org.gnome.Platform//46 org.gnome.Sdk//46
flatpak install flathub org.freedesktop.Sdk.Extension.rust-stable//23.08

git clone https://github.com/rbudiharso/gnome-shot.git
cd gnome-shot
flatpak-builder --user --install --force-clean build-dir org.gnome.GnomeShot.yml

# Run
flatpak run org.gnome.GnomeShot
```

### Build from source

```bash
# Clone the repository
git clone https://github.com/rbudiharso/gnome-shot.git
cd gnome-shot

# Build
cargo build --release

# Install binary (optional)
sudo cp target/release/gnome-shot /usr/local/bin/
```

### Install GNOME Shell Extension

The extension provides a global keyboard shortcut (default: `Shift+Super+S`).

```bash
# Install the extension
./extension/install.sh

# Log out and log back in (required on Wayland)

# Enable the extension
gnome-extensions enable gnome-shot@gnome-shot.github.io
```

## Usage

### Running the app

```bash
# Open the app
gnome-shot

# Open and immediately start capture
gnome-shot --capture
```

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | New capture |
| `Ctrl+S` | Save screenshot |
| `Ctrl+C` | Copy to clipboard |
| `Ctrl+Z` | Undo |
| `Ctrl+Shift+Z` | Redo |
| `Escape` | Quick save & exit (saves to default folder, copies to clipboard) |
| `Ctrl+Q` | Quit |

### Global Shortcut (with extension)

| Shortcut | Action |
|----------|--------|
| `Shift+Super+S` | Launch GNOME Shot and start capture |

## Configuration

### Default save folder

Screenshots are saved to `~/Pictures/Screenshots/` by default.

To change the default folder, create `~/.config/gnome-shot/config`:

```
save_dir=/path/to/your/folder
```

### Extension shortcut

You can customize the global shortcut in GNOME Extensions settings or by editing:
`~/.local/share/gnome-shell/extensions/gnome-shot@gnome-shot.github.io/schemas/`

## License

GPL-3.0-or-later
