#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_DIR/build"
APP_NAME="AfrimDesktop.app"
INSTALL_DIR="$HOME/Library/Input Methods"

if [ ! -d "$BUILD_DIR/$APP_NAME" ]; then
    echo "ERROR: $BUILD_DIR/$APP_NAME not found. Run scripts/build.sh first."
    exit 1
fi

echo "==> Installing to $INSTALL_DIR..."
mkdir -p "$INSTALL_DIR"

# Kill existing instance if running
killall AfrimDesktopInputMethod 2>/dev/null || true

# Copy the app bundle
rm -rf "$INSTALL_DIR/$APP_NAME"
cp -R "$BUILD_DIR/$APP_NAME" "$INSTALL_DIR/"

echo "==> Installed: $INSTALL_DIR/$APP_NAME"
echo ""
echo "Note: macOS input methods must live in ~/Library/Input Methods/."
echo "      They are not picked up from /Applications/."
echo ""
echo "To enable:"
echo "  1. Log out and log back in (or restart)"
echo "  2. System Settings > Keyboard > Input Sources > Edit > +"
echo "  3. Add 'Afrim Desktop' (listed under English)"
echo ""
echo "To try without logging out:"
echo "  killall AfrimDesktopInputMethod 2>/dev/null"
echo "  open '$INSTALL_DIR/$APP_NAME'"
