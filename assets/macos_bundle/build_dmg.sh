#!/bin/bash

set -e

MACOS_BIN_NAME=vds
MACOS_APP_NAME='Volume Data Suite'
MACOS_APP_DIR="$MACOS_APP_NAME.app"

echo "Creating app directory structure"
rm -rf "$MACOS_APP_DIR"
rm -rf *.dmg
rm -rf *.app
rm -rf bundle
mkdir -p "$MACOS_APP_DIR/Contents/MacOS"
mkdir -p "$MACOS_APP_DIR/Contents/Resources"

echo "Setup App Icon"
cp assets/macos_bundle/Info.plist "$MACOS_APP_DIR/Contents/Info.plist"
cp assets/macos_bundle/AppIcon.icns "$MACOS_APP_DIR/Contents/Resources/AppIcon.icns"

echo "Copying binary"
MACOS_APP_BIN="$MACOS_APP_DIR/Contents/MacOS/$MACOS_APP_NAME"
cp target/release/$MACOS_BIN_NAME "$MACOS_APP_BIN"

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
