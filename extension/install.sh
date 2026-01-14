#!/bin/bash

# GNOME Shot Extension Installer

EXTENSION_UUID="gnome-shot@gnome-shot.github.io"
EXTENSION_DIR="$HOME/.local/share/gnome-shell/extensions/$EXTENSION_UUID"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Installing GNOME Shot extension..."

# Create extension directory
mkdir -p "$EXTENSION_DIR"

# Copy extension files
cp "$SCRIPT_DIR/metadata.json" "$EXTENSION_DIR/"
cp "$SCRIPT_DIR/extension.js" "$EXTENSION_DIR/"
cp "$SCRIPT_DIR/prefs.js" "$EXTENSION_DIR/" 2>/dev/null || true

# Copy and compile schemas
mkdir -p "$EXTENSION_DIR/schemas"
cp "$SCRIPT_DIR/schemas/"*.xml "$EXTENSION_DIR/schemas/"
glib-compile-schemas "$EXTENSION_DIR/schemas/"

echo "Extension installed to: $EXTENSION_DIR"
echo ""
echo "To enable the extension:"
echo "  1. Log out and log back in, OR"
echo "  2. Press Alt+F2, type 'r', press Enter (X11 only), OR"
echo "  3. Run: gnome-extensions enable $EXTENSION_UUID"
echo ""
echo "Default shortcut: Shift+Super+S"
echo ""
echo "You can customize the shortcut in GNOME Extensions settings."
