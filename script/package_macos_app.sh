#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "package_macos_app.sh must run on a macOS host" >&2
  exit 1
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_NAME="Honk300"
BUNDLE_ID="dev.emmetts.honk300"
STAGE_DIR="${1:-"$ROOT/target/dist/macos-universal2"}"
APP_DIR="$STAGE_DIR/$APP_NAME.app"
CONTENTS_DIR="$APP_DIR/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"
BIN="$MACOS_DIR/honk300"

cd "$ROOT"

cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

rm -rf "$APP_DIR"
mkdir -p "$MACOS_DIR" "$RESOURCES_DIR"

lipo -create \
  "$ROOT/target/x86_64-apple-darwin/release/honk300" \
  "$ROOT/target/aarch64-apple-darwin/release/honk300" \
  -output "$BIN"
chmod 755 "$BIN"

cp -R "$ROOT/Assets" "$RESOURCES_DIR/Assets"

cat > "$CONTENTS_DIR/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleDisplayName</key>
  <string>Honk300</string>
  <key>CFBundleExecutable</key>
  <string>honk300</string>
  <key>CFBundleIdentifier</key>
  <string>$BUNDLE_ID</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>Honk300</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>0.0.0</string>
  <key>CFBundleVersion</key>
  <string>0</string>
  <key>LSMinimumSystemVersion</key>
  <string>11.0</string>
  <key>LSUIElement</key>
  <true/>
  <key>NSHighResolutionCapable</key>
  <true/>
</dict>
</plist>
PLIST

plutil -lint "$CONTENTS_DIR/Info.plist"
lipo "$BIN" -verify_arch x86_64 arm64
codesign --force --deep --sign - "$APP_DIR"
codesign --verify --deep --strict "$APP_DIR"

echo "Staged $APP_DIR"
