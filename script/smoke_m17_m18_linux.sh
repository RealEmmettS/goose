#!/usr/bin/env sh
set -eu

if [ "$(uname -s)" != "Linux" ]; then
  echo "smoke_m17_m18_linux: must run on Linux" >&2
  exit 1
fi

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
BIN="${ROOT}/target/debug/honk300"
CONFIG="$(mktemp "${TMPDIR:-/tmp}/honk300-linux-config.XXXXXX.toml")"
PID=""

cleanup() {
  "${BIN}" stop >/dev/null 2>&1 || true
  if [ -n "${PID}" ]; then
    wait "${PID}" 2>/dev/null || true
  fi
  rm -f "${CONFIG}" /tmp/honk300-linux-status.txt
}
trap cleanup EXIT INT TERM

wait_for_status() {
  ready=0
  for _ in $(seq 1 80); do
    if "${BIN}" status >/tmp/honk300-linux-status.txt 2>&1; then
      ready=1
      break
    fi
    sleep 0.25
  done
  if [ "${ready}" -ne 1 ]; then
    cat /tmp/honk300-linux-status.txt >&2 || true
    echo "smoke_m17_m18_linux: runtime did not answer status" >&2
    exit 1
  fi
}

exercise_mode() {
  label="$1"
  shift

  echo "smoke_m17_m18_linux: starting ${label}"
  "${BIN}" start --config "${CONFIG}" "$@" &
  PID="$!"
  wait_for_status
  cat /tmp/honk300-linux-status.txt
  grep -q "platform: Linux" /tmp/honk300-linux-status.txt
  grep -Eq "cursor: (unsupported|failed)" /tmp/honk300-linux-status.txt
  grep -Eq "window: (unsupported|failed)" /tmp/honk300-linux-status.txt
  grep -Eq "collect: (unsupported|failed)" /tmp/honk300-linux-status.txt

  "${BIN}" do honk
  "${BIN}" do mud
  "${BIN}" do wander
  if "${BIN}" do nab >/tmp/honk300-linux-nab.txt 2>&1; then
    echo "smoke_m17_m18_linux: nab unexpectedly succeeded in ${label}" >&2
    cat /tmp/honk300-linux-nab.txt >&2
    exit 1
  fi
  grep -q "UNSUPPORTED" /tmp/honk300-linux-nab.txt
  "${BIN}" reload
  "${BIN}" stop
  wait "${PID}" 2>/dev/null || true
  PID=""
}

echo "smoke_m17_m18_linux: building debug binary"
cargo build --manifest-path "${ROOT}/Cargo.toml"

echo "smoke_m17_m18_linux: preparing config"
"${BIN}" setup --config "${CONFIG}"

exercise_mode "default Linux display detection"
exercise_mode "forced Wayland degraded mode" --wayland

echo "smoke_m17_m18_linux: IPC/degraded-mode smoke passed"
echo "smoke_m17_m18_linux: manual follow-up still required for visible X11 overlay/input/window support and visible Wayland layer-shell rendering."
