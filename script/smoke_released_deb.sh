#!/usr/bin/env sh
set -eu
umask 077

TAG="${1:-}"
if ! printf '%s\n' "$TAG" | grep -Eq '^v[0-9]+\.[0-9]+\.[0-9]+$'; then
  printf 'expected a stable vMAJOR.MINOR.PATCH tag\n' >&2
  exit 2
fi
VERSION="${TAG#v}"
case "$(uname -m)" in
  x86_64) ARCHITECTURE=amd64 ;;
  aarch64|arm64) ARCHITECTURE=arm64 ;;
  *) printf 'unsupported Debian smoke architecture: %s\n' "$(uname -m)" >&2; exit 2 ;;
esac

REPOSITORY="https://github.com/RealEmmettS/goose"
PROJECT_ROOT="$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)"
ROOT="$(mktemp -d "${TMPDIR:-/tmp}/honk300-deb-smoke.XXXXXX")"
PACKAGE="$ROOT/honk300-$ARCHITECTURE.deb"
LATEST_PACKAGE="$ROOT/latest-honk300-$ARCHITECTURE.deb"
SIDECAR="$ROOT/honk300-$ARCHITECTURE.deb.sha256"
PACKAGED_TRAY_ICON=/usr/share/icons/hicolor/36x36/apps/honk300.png
trap 'sudo dpkg --remove honk300 >/dev/null 2>&1 || true; rm -rf "$ROOT"' EXIT HUP INT TERM

HOME="$ROOT/home"
XDG_DATA_HOME="$ROOT/data"
XDG_CONFIG_HOME="$ROOT/config"
export HOME XDG_DATA_HOME XDG_CONFIG_HOME
mkdir -p "$HOME" "$XDG_DATA_HOME" "$XDG_CONFIG_HOME"

BASE="$REPOSITORY/releases/download/$TAG"
LOCAL_PACKAGE="${HONK300_DEB_PACKAGE:-}"
SKIP_LATEST="${HONK300_DEB_SKIP_LATEST:-0}"
if [ -n "$LOCAL_PACKAGE" ]; then
  [ "$SKIP_LATEST" = 1 ] || {
    printf 'local Debian smoke input must explicitly skip the unpublished latest comparison\n' >&2
    exit 1
  }
  [ -f "$LOCAL_PACKAGE" ] && [ ! -L "$LOCAL_PACKAGE" ] || {
    printf 'local Debian smoke input is not a regular non-symlink file: %s\n' "$LOCAL_PACKAGE" >&2
    exit 1
  }
  cp "$LOCAL_PACKAGE" "$PACKAGE"
else
  [ "$SKIP_LATEST" = 0 ] || {
    printf 'latest comparison can be skipped only for an explicit local candidate package\n' >&2
    exit 1
  }
  curl --proto '=https' --tlsv1.2 -fsSL "$BASE/honk300-$ARCHITECTURE.deb" -o "$PACKAGE"
  curl --proto '=https' --tlsv1.2 -fsSL "$BASE/honk300-$ARCHITECTURE.deb.sha256" -o "$SIDECAR"
  curl --proto '=https' --tlsv1.2 -fsSL \
    "$REPOSITORY/releases/latest/download/honk300-$ARCHITECTURE.deb" -o "$LATEST_PACKAGE"
  expected="$(awk 'NR == 1 { print $1 }' "$SIDECAR" | tr 'A-F' 'a-f')"
  actual="$(sha256sum "$PACKAGE" | awk '{ print $1 }')"
  [ "$actual" = "$expected" ] || {
    printf 'Debian package checksum mismatch\n' >&2
    exit 1
  }
  cmp "$PACKAGE" "$LATEST_PACKAGE"
fi
actual="$(sha256sum "$PACKAGE" | awk '{ print $1 }')"
[ "$(dpkg-deb --field "$PACKAGE" Package)" = honk300 ]
[ "$(dpkg-deb --field "$PACKAGE" Version)" = "$VERSION" ]
[ "$(dpkg-deb --field "$PACKAGE" Architecture)" = "$ARCHITECTURE" ]
EVIDENCE_DIR="${HONK300_DEB_EVIDENCE_DIR:-}"
[ -n "$EVIDENCE_DIR" ] || {
  printf 'Debian compositor evidence directory is required\n' >&2
  exit 1
}
mkdir -p "$EVIDENCE_DIR"
dpkg-deb --info "$PACKAGE" > "$EVIDENCE_DIR/package-info.txt"
dpkg-deb --contents "$PACKAGE" > "$EVIDENCE_DIR/package-contents.txt"
printf 'package_sha256=%s\n' "$actual" > "$EVIDENCE_DIR/package-identity.txt"

install_and_verify() {
  sudo apt-get install --yes "$PACKAGE"
  [ -x /usr/lib/honk300/honk300 ]
  [ "$(cat /usr/lib/honk300/install-source.txt)" = deb ]
  [ -f "$PACKAGED_TRAY_ICON" ] && [ ! -L "$PACKAGED_TRAY_ICON" ]
  ldd /usr/lib/honk300/honk300 > "$EVIDENCE_DIR/installed-ldd.txt"
  if grep -F 'not found' "$EVIDENCE_DIR/installed-ldd.txt"; then
    printf 'Debian package left a runtime library unresolved\n' >&2
    exit 1
  fi
  for name in honk300 honk goose; do
    [ -L "/usr/bin/$name" ]
    [ "$(readlink "/usr/bin/$name")" = '../lib/honk300/honk300' ]
    reported="$("/usr/bin/$name" --version | awk '{ print $NF }' | sed 's/[+-].*$//')"
    [ "$reported" = "$VERSION" ]
    "/usr/bin/$name" update | grep -F "already on the latest version ($VERSION)"
  done
  dpkg-query --search /usr/lib/honk300/honk300 | grep -F 'honk300: /usr/lib/honk300/honk300'
}

install_and_verify
binary_before="$(sha256sum /usr/lib/honk300/honk300 | awk '{ print $1 }')"
HONK300_BIN=/usr/lib/honk300/honk300 \
HONK300_EVIDENCE_DIR="$EVIDENCE_DIR" \
  sh "$PROJECT_ROOT/script/smoke_m17_m18_linux.sh"
binary_after="$(sha256sum /usr/lib/honk300/honk300 | awk '{ print $1 }')"
[ "$binary_after" = "$binary_before" ]
{
  printf 'installed_binary_sha256_before=%s\n' "$binary_before"
  printf 'installed_binary_sha256_after=%s\n' "$binary_after"
  printf 'package_sha256=%s\n' "$actual"
} > "$EVIDENCE_DIR/debian-installed-identity.txt"

mkdir -p "$XDG_DATA_HOME/honk300/media/Notes"
printf 'keep me\n' > "$XDG_DATA_HOME/honk300/media/Notes/user-note.txt"
/usr/bin/goose uninstall
if dpkg-query --show honk300 >/dev/null 2>&1; then
  printf 'Debian package remained installed after normal CLI uninstall\n' >&2
  exit 1
fi
[ -f "$XDG_DATA_HOME/honk300/media/Notes/user-note.txt" ]
for name in honk300 honk goose; do [ ! -e "/usr/bin/$name" ]; done
[ ! -e "$PACKAGED_TRAY_ICON" ] && [ ! -L "$PACKAGED_TRAY_ICON" ]

install_and_verify
/usr/bin/honk300 uninstall --purge
if dpkg-query --show honk300 >/dev/null 2>&1; then
  printf 'Debian package remained installed after purge CLI uninstall\n' >&2
  exit 1
fi
[ ! -d "$XDG_DATA_HOME/honk300" ]
find "$XDG_DATA_HOME/honk300-backups" -type f -name user-note.txt -print -quit | grep -q .
for name in honk300 honk goose; do [ ! -e "/usr/bin/$name" ]; done
[ ! -e "$PACKAGED_TRAY_ICON" ] && [ ! -L "$PACKAGED_TRAY_ICON" ]

printf 'published Debian %s %s install, aliases, compositor, update, uninstall, and purge passed\n' \
  "$ARCHITECTURE" "$TAG"
