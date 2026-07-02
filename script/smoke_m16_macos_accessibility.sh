#!/usr/bin/env sh
set -eu

if [ "$(uname -s)" != "Darwin" ]; then
  echo "smoke_m16_macos_accessibility: must run on macOS" >&2
  exit 1
fi

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
APP="${ROOT}/target/dist/macos-universal2/Honk300.app"
BIN="${APP}/Contents/MacOS/honk300"
CONFIG="$(mktemp "${TMPDIR:-/tmp}/honk300-m16-a11y-config.XXXXXX.toml")"
STATUS="$(mktemp "${TMPDIR:-/tmp}/honk300-m16-a11y-status.XXXXXX.txt")"

cleanup() {
  "${BIN}" stop >/dev/null 2>&1 || true
  rm -f "${CONFIG}" "${STATUS}"
}
trap cleanup EXIT INT TERM

start_runtime() {
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
    echo "smoke_m16_macos_accessibility: runtime did not answer status" >&2
    exit 1
  fi
}

stop_runtime() {
  "${BIN}" stop >/dev/null 2>&1 || true
}

exercise_single_action() {
  action="$1"
  start_runtime
  cat "${STATUS}"
  grep -q "platform: macOS" "${STATUS}"
  grep -q "accessibility: supported" "${STATUS}"
  grep -q "cursor: supported" "${STATUS}"
  "${BIN}" do "${action}"
  stop_runtime
}

echo "smoke_m16_macos_accessibility: building universal2 app"
bash "${ROOT}/script/package_macos_app.sh"

"${BIN}" setup --config "${CONFIG}"

start_runtime
cat "${STATUS}"
grep -q "platform: macOS" "${STATUS}"
grep -q "accessibility: supported" "${STATUS}"
grep -q "cursor: supported" "${STATUS}"

"${BIN}" do honk
"${BIN}" do mud
"${BIN}" reload
stop_runtime

exercise_single_action nab
exercise_single_action meme
exercise_single_action note

echo "smoke_m16_macos_accessibility: Accessibility-granted command smoke passed"
