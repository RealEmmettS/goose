#!/usr/bin/env sh
set -eu

if [ "$(uname -s)" != "Darwin" ]; then
  echo "smoke_m16_macos: must run on macOS" >&2
  exit 1
fi

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
APP="${ROOT}/target/dist/macos-universal2/Honk300.app"
BIN="${APP}/Contents/MacOS/honk300"
CONFIG="$(mktemp "${TMPDIR:-/tmp}/honk300-m16-config.XXXXXX.toml")"

cleanup() {
  "${BIN}" stop >/dev/null 2>&1 || true
  rm -f "${CONFIG}"
}
trap cleanup EXIT INT TERM

echo "smoke_m16_macos: building universal2 app"
bash "${ROOT}/script/package_macos_app.sh"

echo "smoke_m16_macos: validating bundle"
plutil -lint "${APP}/Contents/Info.plist"
test "$(plutil -extract CFBundleIdentifier raw "${APP}/Contents/Info.plist")" = "dev.emmetts.honk300"
test "$(plutil -extract LSUIElement raw "${APP}/Contents/Info.plist")" = "1"
codesign --verify --deep --strict "${APP}"
lipo -verify_arch x86_64 arm64 "${BIN}"

echo "smoke_m16_macos: preparing config"
"${BIN}" setup --config "${CONFIG}"

echo "smoke_m16_macos: launching bundled LSUIElement runtime"
/usr/bin/open -n "${APP}" --args start --config "${CONFIG}"

ready=0
for _ in $(seq 1 80); do
  if "${BIN}" status >/tmp/honk300-m16-status.txt 2>&1 && grep -q "honk300: running" /tmp/honk300-m16-status.txt; then
    ready=1
    break
  fi
  sleep 0.25
done
if [ "${ready}" -ne 1 ]; then
  cat /tmp/honk300-m16-status.txt >&2 || true
  echo "smoke_m16_macos: runtime did not answer status" >&2
  exit 1
fi

echo "smoke_m16_macos: status"
cat /tmp/honk300-m16-status.txt
grep -q "platform: macOS" /tmp/honk300-m16-status.txt
grep -Eq "accessibility: (supported|denied)" /tmp/honk300-m16-status.txt

echo "smoke_m16_macos: exercising IPC"
"${BIN}" do honk
"${BIN}" do mud
"${BIN}" reload
"${BIN}" stop

echo "smoke_m16_macos: automated bundle/status smoke passed"
echo "smoke_m16_macos: manual follow-up still required for granted Accessibility, foreign-window ride, collect note/meme, terminal non-targeting, and multi-monitor behavior."
