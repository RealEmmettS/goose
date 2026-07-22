#!/usr/bin/env sh
set -eu
umask 077

# Native staged proof for the tray helper's real Updated path. Build the current source with an
# older fixture version, install it as a genuine Debian package, then let that exact helper update
# itself from the official latest manifest. dpkg replaces the running image, so this also proves
# the post-activation fixed-path owner resolver rather than merely testing a no-op.

PROJECT_ROOT="$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)"
ROOT="$(mktemp -d "${TMPDIR:-/tmp}/honk300-helper-updated.XXXXXX")"
SOURCE="$ROOT/source"
TARGET_DIR="$PROJECT_ROOT/target/control-surface-updated-fixture"
EVIDENCE_DIR="${HONK300_EVIDENCE_DIR:-$PROJECT_ROOT/target/control-surface-updated-evidence}"
FIXTURE_VERSION=1.3.2
FIXTURE_TAG="v$FIXTURE_VERSION"
HELPER_PID=""
RUNTIME_PID=""

cleanup() {
  if [ -n "$HELPER_PID" ]; then
    kill "$HELPER_PID" >/dev/null 2>&1 || true
    wait "$HELPER_PID" >/dev/null 2>&1 || true
  fi
  if [ -x /usr/lib/honk300/honk300 ]; then
    /usr/lib/honk300/honk300 stop --force >/dev/null 2>&1 || true
  fi
  if [ -n "$RUNTIME_PID" ]; then
    kill "$RUNTIME_PID" >/dev/null 2>&1 || true
    wait "$RUNTIME_PID" >/dev/null 2>&1 || true
  fi
  sudo dpkg --remove honk300 >/dev/null 2>&1 || true
  rm -rf "$ROOT"
}
trap cleanup EXIT HUP INT TERM

mkdir -p "$SOURCE" "$EVIDENCE_DIR" "$ROOT/home" "$ROOT/runtime"
chmod 700 "$ROOT/runtime"
printf 'source_commit=%s\n' "$(git -C "$PROJECT_ROOT" rev-parse HEAD)" \
  >"$EVIDENCE_DIR/fixture-input.txt"
git -C "$PROJECT_ROOT" archive --format=tar HEAD | tar -xf - -C "$SOURCE"

python3 - "$SOURCE/Cargo.toml" "$SOURCE/Cargo.lock" "$FIXTURE_VERSION" <<'PY'
import pathlib
import re
import sys

manifest, lock, version = map(pathlib.Path, sys.argv[1:])
version = str(version)
manifest_text, count = re.subn(
    r'(?m)^version = "[0-9]+\.[0-9]+\.[0-9]+"$',
    f'version = "{version}"',
    manifest.read_text(),
    count=1,
)
if count != 1:
    raise SystemExit("could not stamp the fixture Cargo manifest")
lock_text, count = re.subn(
    r'(\[\[package\]\]\nname = "honk300"\nversion = ")[^"]+("\n)',
    rf'\g<1>{version}\2',
    lock.read_text(),
    count=1,
)
if count != 1:
    raise SystemExit("could not stamp the fixture Cargo lock")
manifest.write_text(manifest_text)
lock.write_text(lock_text)
PY

CARGO_TARGET_DIR="$TARGET_DIR" cargo build \
  --manifest-path "$SOURCE/Cargo.toml" --locked --bin honk300
FIXTURE_BINARY="$TARGET_DIR/debug/honk300"
test -x "$FIXTURE_BINARY"
test "$("$FIXTURE_BINARY" --version | awk '{ print $NF }')" = "$FIXTURE_VERSION"

COMMIT="$(git -C "$PROJECT_ROOT" rev-parse HEAD)"
SOURCE_DATE_EPOCH="$(git -C "$PROJECT_ROOT" show -s --format=%ct HEAD)"
PACKAGE="$ROOT/honk300-fixture-amd64.deb"
(
  # dpkg-deb requires DEBIAN to be traversable. Keep the outer fixture private while letting the
  # deterministic package builder create ordinary 0755 package directories.
  umask 022
  python3 "$PROJECT_ROOT/script/package_deb.py" \
    --binary "$FIXTURE_BINARY" \
    --output "$PACKAGE" \
    --version "$FIXTURE_VERSION" \
    --tag "$FIXTURE_TAG" \
    --commit "$COMMIT" \
    --architecture amd64 \
    --source-date-epoch "$SOURCE_DATE_EPOCH"
)
sudo apt-get install --yes "$PACKAGE"

export HOME="$ROOT/home"
export XDG_DATA_HOME="$ROOT/data"
export XDG_CONFIG_HOME="$ROOT/config"
export XDG_RUNTIME_DIR="$ROOT/runtime"
# This fixture qualifies update/lifecycle integration; the adjacent Linux smoke owns real
# compositor pixels. Keeping this process headless avoids making helper correctness depend on a
# second compositor inside xvfb-run while still exercising the real singleton and IPC endpoints.
export HONK300_ALLOW_HEADLESS=1
mkdir -p "$XDG_DATA_HOME" "$XDG_CONFIG_HOME"

INSTALLED=/usr/lib/honk300/honk300
test "$("$INSTALLED" --version | awk '{ print $NF }')" = "$FIXTURE_VERSION"
"$INSTALLED" start >"$EVIDENCE_DIR/old-runtime.log" 2>&1 &
RUNTIME_PID=$!
ready_deadline=$(($(date +%s) + 30))
until "$INSTALLED" status 2>/dev/null | grep -F 'honk300: running' >/dev/null; do
  kill -0 "$RUNTIME_PID" >/dev/null 2>&1 || {
    cat "$EVIDENCE_DIR/old-runtime.log" >&2
    echo 'fixture runtime exited before readiness' >&2
    exit 1
  }
  test "$(date +%s)" -lt "$ready_deadline" || {
    echo 'fixture runtime did not become ready' >&2
    exit 1
  }
  sleep 0.1
done

"$INSTALLED" __control-surface-update \
  >"$EVIDENCE_DIR/helper.stdout.txt" \
  2>"$EVIDENCE_DIR/helper.stderr.txt" &
HELPER_PID=$!
update_deadline=$(($(date +%s) + 300))
until grep -F 'Update complete.' "$EVIDENCE_DIR/helper.stdout.txt" >/dev/null 2>&1; do
  kill -0 "$HELPER_PID" >/dev/null 2>&1 || {
    cat "$EVIDENCE_DIR/helper.stderr.txt" >&2
    echo 'fixture update helper exited before its retained Updated result' >&2
    exit 1
  }
  test "$(date +%s)" -lt "$update_deadline" || {
    cat "$EVIDENCE_DIR/helper.stderr.txt" >&2
    echo 'fixture update helper timed out' >&2
    exit 1
  }
  sleep 0.25
done

grep -F 'Honk300 has restarted.' "$EVIDENCE_DIR/helper.stdout.txt" >/dev/null
grep -F 'You may now close this window.' "$EVIDENCE_DIR/helper.stdout.txt" >/dev/null
kill -0 "$HELPER_PID"
UPDATED_VERSION="$("$INSTALLED" --version | awk '{ print $NF }')"
test "$UPDATED_VERSION" != "$FIXTURE_VERSION"
"$INSTALLED" status | grep -F 'honk300: running' >/dev/null
dpkg-query --search "$INSTALLED" | grep -Fx "honk300: $INSTALLED" >/dev/null
python3 - "$UPDATED_VERSION" <<'PY'
import json
import pathlib
import sys

version = sys.argv[1]
receipt = json.loads(pathlib.Path('/usr/lib/honk300/install-receipt.json').read_text())
assert receipt['schema'] == 'honk300.install.v2'
assert receipt['version'] == version
assert receipt['origin'] == 'deb'
assert receipt['installer_family'] == 'deb'
assert receipt['active_release'] == '/usr/lib/honk300'
PY

{
  printf 'fixture_version=%s\n' "$FIXTURE_VERSION"
  printf 'updated_version=%s\n' "$UPDATED_VERSION"
  printf 'helper_pid=%s\n' "$HELPER_PID"
  printf 'result=updated_restarted_ready_and_held\n'
} >"$EVIDENCE_DIR/result.txt"

printf 'native Debian helper Updated/restart/hold fixture passed (%s -> %s)\n' \
  "$FIXTURE_VERSION" "$UPDATED_VERSION"
