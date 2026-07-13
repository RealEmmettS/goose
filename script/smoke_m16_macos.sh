#!/usr/bin/env sh
set -eu

if [ "$(uname -s)" != "Darwin" ]; then
  echo "smoke_m16_macos: must run on macOS" >&2
  exit 1
fi

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
APP="${HONK300_APP:-${ROOT}/target/dist/macos-universal2/Honk300.app}"
BIN="${APP}/Contents/MacOS/honk300"
TEMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/honk300-m16.XXXXXX")"
CONFIG="${TEMP_ROOT}/config.toml"
STATUS="${TEMP_ROOT}/status.txt"

cleanup() {
  if [ -x "${BIN}" ]; then
    "${BIN}" stop >/dev/null 2>&1 || true
  fi
  rm -rf "${TEMP_ROOT}"
}
trap cleanup EXIT INT TERM

if [ "${HONK300_SKIP_BUILD:-0}" = "1" ]; then
  echo "smoke_m16_macos: using exact prebuilt app ${APP}"
else
  echo "smoke_m16_macos: building universal2 app"
  bash "${ROOT}/script/package_macos_app.sh"
fi

if [ ! -d "${APP}" ] || [ ! -x "${BIN}" ]; then
  echo "smoke_m16_macos: app bundle or executable is missing: ${APP}" >&2
  exit 1
fi

echo "smoke_m16_macos: validating bundle"
plutil -lint "${APP}/Contents/Info.plist"
test "$(plutil -extract CFBundleIdentifier raw "${APP}/Contents/Info.plist")" = "dev.emmetts.honk300"
LSUI_ELEMENT="$(plutil -extract LSUIElement raw "${APP}/Contents/Info.plist")"
case "${LSUI_ELEMENT}" in
  1|true) ;;
  *)
    echo "smoke_m16_macos: expected LSUIElement true, got ${LSUI_ELEMENT}" >&2
    exit 1
    ;;
esac
codesign --verify --strict "${APP}"
lipo "${BIN}" -verify_arch x86_64 arm64

echo "smoke_m16_macos: preparing config"
"${BIN}" setup --config "${CONFIG}"

echo "smoke_m16_macos: launching bundled LSUIElement runtime"
/usr/bin/open -n "${APP}" --args start --config "${CONFIG}"

ready=0
for _ in $(seq 1 80); do
  if "${BIN}" status >"${STATUS}" 2>&1 && grep -q "honk300: running" "${STATUS}"; then
    ready=1
    break
  fi
  sleep 0.25
done
if [ "${ready}" -ne 1 ]; then
  cat "${STATUS}" >&2 || true
  echo "smoke_m16_macos: runtime did not answer status" >&2
  exit 1
fi

echo "smoke_m16_macos: status"
cat "${STATUS}"
grep -q "platform: macOS" "${STATUS}"
grep -Eq "accessibility: (supported|denied)" "${STATUS}"

echo "smoke_m16_macos: exercising IPC"
"${BIN}" do honk
"${BIN}" do mud
"${BIN}" reload
"${BIN}" stop

echo "smoke_m16_macos: automated bundle/status smoke passed"
echo "smoke_m16_macos: manual follow-up still required for granted Accessibility, foreign-window ride, collect note/meme, terminal non-targeting, and multi-monitor behavior."
