#!/bin/bash

set -e

MACOS_BIN_NAME=vds
MACOS_APP_NAME='Volume Data Suite'
MACOS_APP_DIR="$MACOS_APP_NAME.app"

# Check if the file "vds" exists in the target/release directory
if [ -f "target/release/vds" ]; then
    MACOS_BIN_PATH="target/release"
# Check if the file "vds" exists in the target/x86_64-apple-darwin/release directory
elif [ -f "target/x86_64-apple-darwin/release/vds" ]; then
    MACOS_BIN_PATH="target/x86_64-apple-darwin/release"
# Check if the file "vds" exists in the target/aarch64-apple-darwin/release directory
elif [ -f "target/aarch64-apple-darwin/release/vds" ]; then
    MACOS_BIN_PATH="target/aarch64-apple-darwin/release"
else
    echo "Error: The file 'vds' was not found in any of the specified paths."
    exit 1
fi

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
cp $MACOS_BIN_PATH/$MACOS_BIN_NAME "$MACOS_APP_BIN"
