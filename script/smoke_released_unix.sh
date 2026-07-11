#!/bin/sh
# Destructive only inside a temporary HOME/XDG tree. Exercises immutable live release assets.

set -eu
umask 077

TAG="${1:-}"
if ! printf '%s\n' "$TAG" | grep -Eq '^v[0-9]+\.[0-9]+\.[0-9]+$'; then
  printf 'expected a stable vMAJOR.MINOR.PATCH tag\n' >&2
  exit 2
fi
VERSION="${TAG#v}"
REPOSITORY="https://github.com/RealEmmettS/goose"
ROOT="$(mktemp -d "${TMPDIR:-/tmp}/honk300-live-smoke.XXXXXX")"
trap 'rm -rf "$ROOT"' EXIT HUP INT TERM

file_sha256() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

require_file() {
  [ -f "$1" ] && [ ! -L "$1" ] || { printf 'missing regular file: %s\n' "$1" >&2; exit 1; }
}

HOME="$ROOT/home"
XDG_DATA_HOME="$ROOT/data"
XDG_CONFIG_HOME="$ROOT/config"
export HOME XDG_DATA_HOME XDG_CONFIG_HOME
mkdir -p "$HOME" "$XDG_DATA_HOME" "$XDG_CONFIG_HOME"
PATH="/usr/bin:/bin:/usr/sbin:/sbin"
export PATH

OS="$(uname -s)"
case "$OS" in
  Darwin)
    SHELL=/bin/zsh
    PROFILE="$HOME/.zprofile"
    DEST="$HOME/Applications/Honk300.app"
    BINARY="$DEST/Contents/MacOS/honk300"
    RECEIPT="$HOME/Library/Application Support/honk300/install-receipt.json"
    AUTOSTART="$HOME/Library/LaunchAgents/dev.emmetts.honk300.plist"
    DESKTOP=""
    ;;
  Linux)
    SHELL=/bin/sh
    PROFILE="$HOME/.profile"
    DEST="$XDG_DATA_HOME/honk300/install"
    BINARY="$DEST/bin/honk300"
    RECEIPT="$XDG_DATA_HOME/honk300/install-receipt.json"
    AUTOSTART="$XDG_CONFIG_HOME/autostart/honk300.desktop"
    DESKTOP="$XDG_DATA_HOME/applications/honk300.desktop"
    ;;
  *) printf 'unsupported smoke host: %s\n' "$OS" >&2; exit 2 ;;
esac
export SHELL

printf 'user-profile-line\n' > "$PROFILE"
mkdir -p "$(dirname "$AUTOSTART")"
printf 'foreign-autostart-state\n' > "$AUTOSTART"
AUTOSTART_BASELINE="$ROOT/autostart.baseline"
cp "$AUTOSTART" "$AUTOSTART_BASELINE"

INSTALLER="$ROOT/honk300-installer.sh"
SIDECAR="$ROOT/honk300-installer.sh.sha256"
BASE="$REPOSITORY/releases/download/$TAG"
curl --proto '=https' --tlsv1.2 -fsSL "$BASE/honk300-installer.sh" -o "$INSTALLER"
curl --proto '=https' --tlsv1.2 -fsSL "$BASE/honk300-installer.sh.sha256" -o "$SIDECAR"
expected="$(awk 'NR == 1 { print $1 }' "$SIDECAR" | tr 'A-F' 'a-f')"
actual="$(file_sha256 "$INSTALLER" | tr 'A-F' 'a-f')"
[ "$actual" = "$expected" ] || { printf 'installer checksum mismatch\n' >&2; exit 1; }
grep -F "TAG=\"$TAG\"" "$INSTALLER" >/dev/null
grep -F "VERSION=\"$VERSION\"" "$INSTALLER" >/dev/null

verify_install() {
  [ -x "$BINARY" ] || { printf 'installed binary is missing: %s\n' "$BINARY" >&2; exit 1; }
  reported="$($BINARY --version | awk '{print $NF}' | sed 's/[+-].*$//')"
  [ "$reported" = "$VERSION" ] || { printf 'installed version mismatch: %s\n' "$reported" >&2; exit 1; }
  require_file "$RECEIPT"
  grep -F '"schema": "honk300.install.v1"' "$RECEIPT" >/dev/null
  grep -F "\"tag\": \"$TAG\"" "$RECEIPT" >/dev/null
  grep -F '"autostart": { "enabled": true, "owner": "honk300-installer" }' "$RECEIPT" >/dev/null

  for link in \
    "$HOME/.local/bin/honk300" \
    "$HOME/.local/bin/honk" \
    "$HOME/.local/bin/goose"
  do
    [ -L "$link" ] || { printf 'managed alias is missing: %s\n' "$link" >&2; exit 1; }
    [ "$(readlink "$link")" = "$BINARY" ] || { printf 'managed alias target changed: %s\n' "$link" >&2; exit 1; }
  done

  [ "$(grep -c '^# >>> honk300 managed PATH >>>$' "$PROFILE")" = 1 ] || {
    printf 'managed PATH marker is missing or duplicated\n' >&2
    exit 1
  }
  grep -F 'user-profile-line' "$PROFILE" >/dev/null
  cmp "$AUTOSTART_BASELINE" "$AUTOSTART"

  if [ "$OS" = Linux ]; then
    require_file "$DESKTOP"
    grep -F 'X-Honk300-Managed=true' "$DESKTOP" >/dev/null
    grep -F "Exec=$BINARY start" "$DESKTOP" >/dev/null
  else
    codesign --verify --deep --strict "$DEST"
    lipo "$BINARY" -verify_arch x86_64 arm64
    [ "$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$DEST/Contents/Info.plist")" = dev.emmetts.honk300 ]
    [ "$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$DEST/Contents/Info.plist")" = "$VERSION" ]
  fi
}

snapshot_state() {
  output="$1"
  {
    printf 'binary %s\n' "$(file_sha256 "$BINARY")"
    printf 'receipt %s\n' "$(file_sha256 "$RECEIPT")"
    printf 'profile %s\n' "$(file_sha256 "$PROFILE")"
    printf 'autostart %s\n' "$(file_sha256 "$AUTOSTART")"
    for link in \
      "$HOME/.local/bin/honk300" \
      "$HOME/.local/bin/honk" \
      "$HOME/.local/bin/goose"
    do
      printf 'alias %s %s\n' "$link" "$(readlink "$link")"
    done
    if [ "$OS" = Linux ]; then
      printf 'desktop %s\n' "$(file_sha256 "$DESKTOP")"
    else
      printf 'info %s\n' "$(file_sha256 "$DEST/Contents/Info.plist")"
      printf 'signature %s\n' "$(file_sha256 "$DEST/Contents/_CodeSignature/CodeResources")"
    fi
  } > "$output"
}

for pass in 1 2; do
  sh "$INSTALLER"
  verify_install
done

BEFORE_STATE="$ROOT/before.state"
AFTER_STATE="$ROOT/after.state"
snapshot_state "$BEFORE_STATE"

set +e
HONK300_TEST_FAIL_AFTER_SWAP=1 sh "$INSTALLER" > "$ROOT/fault.log" 2>&1
fault_status=$?
set -e
[ "$fault_status" -ne 0 ] || { printf 'fault injection unexpectedly succeeded\n' >&2; exit 1; }

verify_install
snapshot_state "$AFTER_STATE"
if ! cmp "$BEFORE_STATE" "$AFTER_STATE"; then
  printf 'complete managed integration state was not restored after fault\n' >&2
  cat "$ROOT/fault.log" >&2
  exit 1
fi

for stale in "$DEST.previous."*; do
  if [ -e "$stale" ] || [ -L "$stale" ]; then
    printf 'rollback left a previous-install directory behind: %s\n' "$stale" >&2
    exit 1
  fi
done

printf 'live %s %s install-twice and rollback smoke passed\n' "$OS" "$TAG"
