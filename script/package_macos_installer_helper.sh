#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "package_macos_installer_helper.sh must run on a macOS host" >&2
  exit 1
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="${HONK300_VERSION:-0.0.0}"
IDENTITY="${MACOS_SIGN_IDENTITY:--}"
STAGE_DIR="${1:-"$ROOT/target/dist/macos-universal2"}"
APP_DIR="$STAGE_DIR/Install Honk300.app"
CONTENTS="$APP_DIR/Contents"
EXECUTABLE="$CONTENTS/MacOS/Install Honk300"
BUILD_DIR=""

[[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || {
  echo "invalid helper bundle version: $VERSION" >&2
  exit 1
}

rm -rf "$APP_DIR"
mkdir -p "$CONTENTS/MacOS"
BUILD_DIR="$(mktemp -d "${TMPDIR:-/tmp}/honk300-installer-helper.XXXXXX")"
trap 'rm -rf "$BUILD_DIR"' EXIT
for target in x86_64-apple-macos11.0 arm64-apple-macos11.0; do
  xcrun swiftc \
    -target "$target" \
    -parse-as-library \
    -O \
    -framework AppKit \
    -framework Security \
    "$ROOT/packaging/macos/InstallHonk300/main.swift" \
    -o "$BUILD_DIR/$target"
done
lipo -create \
  "$BUILD_DIR/x86_64-apple-macos11.0" \
  "$BUILD_DIR/arm64-apple-macos11.0" \
  -output "$EXECUTABLE"
lipo "$EXECUTABLE" -verify_arch x86_64 arm64
xcrun vtool -show-build "$EXECUTABLE" > "$BUILD_DIR/helper-deployment.txt"
test "$(grep -c 'minos 11.0' "$BUILD_DIR/helper-deployment.txt")" -eq 2

cat > "$CONTENTS/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleDisplayName</key>
  <string>Install Honk300</string>
  <key>CFBundleExecutable</key>
  <string>Install Honk300</string>
  <key>CFBundleIdentifier</key>
  <string>dev.emmetts.honk300.installer</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>Install Honk300</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>$VERSION</string>
  <key>CFBundleVersion</key>
  <string>$VERSION</string>
  <key>LSMinimumSystemVersion</key>
  <string>11.0</string>
</dict>
</plist>
PLIST

plutil -lint "$CONTENTS/Info.plist"
if [[ "$IDENTITY" == "-" ]]; then
  codesign --force --options runtime --sign - "$EXECUTABLE"
  codesign --force --options runtime --sign - "$APP_DIR"
else
  codesign --force --options runtime --timestamp --sign "$IDENTITY" "$EXECUTABLE"
  codesign --force --options runtime --timestamp --sign "$IDENTITY" "$APP_DIR"
fi
codesign --verify --strict --verbose=2 "$EXECUTABLE"
codesign --verify --strict --verbose=2 "$APP_DIR"

rm -rf "$BUILD_DIR"
trap - EXIT
echo "Staged $APP_DIR"
