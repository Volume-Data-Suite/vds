#!/bin/bash

set -e

MACOS_APP_NAME='Volume Data Suite'
MACOS_APP_DIR="$MACOS_APP_NAME.app"

echo "Collecting content for DMG file"
mkdir -p bundle
mv "$MACOS_APP_DIR" bundle/"$MACOS_APP_DIR"

create-dmg \
    --volname "$MACOS_APP_NAME" \
    --background assets/macos_bundle/DiskImageBackground.png \
    --window-pos 200 120 \
    --window-size 800 400 \
    --volicon assets/macos_bundle/DiskImageIcon.icns \
    --icon-size 200 \
    --icon "$MACOS_APP_DIR" 200 190 \
    --hide-extension "$MACOS_APP_DIR" \
    --app-drop-link 600 185 \
    "$MACOS_APP_NAME.dmg" bundle
