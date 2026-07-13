#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "package_macos_app.sh must run on a macOS host" >&2
  exit 1
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_NAME="Honk300"
BUNDLE_ID="dev.emmetts.honk300"
# Stamp the bundle version from HONK300_VERSION (the release workflow passes the resolved tag
# without its leading `v`); default to 0.0.0 for local/unversioned staging.
VERSION="${HONK300_VERSION:-0.0.0}"
TAG="${HONK300_TAG:-v$VERSION}"
COMMIT="${HONK300_COMMIT:-$(git -C "$ROOT" rev-parse HEAD)}"
IDENTITY="${MACOS_SIGN_IDENTITY:--}"
[[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || {
  echo "invalid bundle version: $VERSION" >&2
  exit 1
}
[[ "$TAG" == "v$VERSION" ]] || {
  echo "bundle tag $TAG does not match version $VERSION" >&2
  exit 1
}
[[ "$COMMIT" =~ ^[0-9a-fA-F]{40}$ ]] || {
  echo "bundle commit is not a full hexadecimal SHA" >&2
  exit 1
}
COMMIT="$(printf '%s' "$COMMIT" | tr '[:upper:]' '[:lower:]')"
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

ditto "$ROOT/LICENSE" "$RESOURCES_DIR/LICENSE"
ditto "$ROOT/THIRD_PARTY_ASSETS.md" "$RESOURCES_DIR/THIRD_PARTY_ASSETS.md"

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
  <string>$VERSION</string>
  <key>CFBundleVersion</key>
  <string>$VERSION</string>
  <key>Honk300ReleaseTag</key>
  <string>$TAG</string>
  <key>Honk300ReleaseCommit</key>
  <string>$COMMIT</string>
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
if [[ "$IDENTITY" == "-" ]]; then
  codesign --force --options runtime --sign - "$BIN"
  codesign --force --options runtime --sign - "$APP_DIR"
else
  codesign --force --options runtime --timestamp --sign "$IDENTITY" "$BIN"
  codesign --force --options runtime --timestamp --sign "$IDENTITY" "$APP_DIR"
fi
codesign --verify --strict "$BIN"
codesign --verify --strict "$APP_DIR"

echo "Staged $APP_DIR"
