#!/usr/bin/env sh
# shellcheck disable=SC1007,SC1010
set -eu

if [ "$(uname -s)" != "Linux" ]; then
  echo "smoke_m17_m18_linux: must run on Linux" >&2
  exit 1
fi

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
ANALYZER="${ROOT}/script/analyze_linux_overlay_capture.py"
if [ -n "${HONK300_BIN:-}" ]; then
  BIN="${HONK300_BIN}"
  BUILD_BIN=0
else
  BIN="${ROOT}/target/debug/honk300"
  BUILD_BIN=1
fi
EVIDENCE_DIR="${HONK300_EVIDENCE_DIR:-}"
if [ -n "${EVIDENCE_DIR}" ]; then
  mkdir -p "${EVIDENCE_DIR}"
  WORK="${EVIDENCE_DIR}/run"
  rm -rf "${WORK}"
  mkdir -p "${WORK}"
else
  WORK="$(mktemp -d "${TMPDIR:-/tmp}/honk300-linux-smoke.XXXXXX")"
fi
CONFIG="${WORK}/config.toml"
STATUS="${WORK}/status.txt"
NAB="${WORK}/nab.txt"
PID=""
XVFB_PID=""
OPENBOX_PID=""
XCOMPMGR_PID=""
X11_BACKGROUND_PID=""
SWAY_PID=""
WAYLAND_RUNTIME_DIR=""
RUNTIME_PAUSED=0
WAYLAND_FIRST_OUTPUT=""
WAYLAND_SECOND_OUTPUT=""

cleanup() {
  if [ "${RUNTIME_PAUSED}" -eq 1 ] && [ -n "${PID}" ]; then
    kill -CONT "${PID}" >/dev/null 2>&1 || true
    RUNTIME_PAUSED=0
  fi
  "${BIN}" stop >/dev/null 2>&1 || true
  if [ -n "${PID}" ]; then
    wait "${PID}" 2>/dev/null || true
  fi
  if [ -n "${OPENBOX_PID}" ]; then
    kill "${OPENBOX_PID}" >/dev/null 2>&1 || true
  fi
  if [ -n "${X11_BACKGROUND_PID}" ]; then
    kill "${X11_BACKGROUND_PID}" >/dev/null 2>&1 || true
    wait "${X11_BACKGROUND_PID}" 2>/dev/null || true
  fi
  if [ -n "${XCOMPMGR_PID}" ]; then
    kill "${XCOMPMGR_PID}" >/dev/null 2>&1 || true
  fi
  if [ -n "${XVFB_PID}" ]; then
    kill "${XVFB_PID}" >/dev/null 2>&1 || true
  fi
  if [ -n "${SWAY_PID}" ]; then
    kill "${SWAY_PID}" >/dev/null 2>&1 || true
  fi
  if [ -n "${WAYLAND_RUNTIME_DIR}" ]; then
    rm -rf "${WAYLAND_RUNTIME_DIR}"
  fi
  if [ -z "${EVIDENCE_DIR}" ]; then
    rm -rf "${WORK}"
  fi
}
trap cleanup EXIT INT TERM

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "smoke_m17_m18_linux: missing required command: $1" >&2
    exit 1
  fi
}

file_sha256() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

pause_runtime() {
  kill -STOP "${PID}"
  RUNTIME_PAUSED=1
  sleep 0.25
}

resume_runtime() {
  kill -CONT "${PID}"
  RUNTIME_PAUSED=0
}

wait_for_status() {
  runtime_log="$1"
  ready=0
  for _ in $(seq 1 100); do
    if "${BIN}" status >"${STATUS}" 2>&1 && grep -q "honk300: running" "${STATUS}"; then
      ready=1
      break
    fi
    sleep 0.25
  done
  if [ "${ready}" -ne 1 ]; then
    cat "${runtime_log}" >&2 || true
    cat "${STATUS}" >&2 || true
    echo "smoke_m17_m18_linux: runtime did not answer status" >&2
    exit 1
  fi
}

wait_for_x11_compositor() {
  for _ in $(seq 1 40); do
    if python3 <<'PY'
import ctypes

x11 = ctypes.CDLL("libX11.so.6")
x11.XOpenDisplay.argtypes = [ctypes.c_char_p]
x11.XOpenDisplay.restype = ctypes.c_void_p
x11.XDefaultScreen.argtypes = [ctypes.c_void_p]
x11.XDefaultScreen.restype = ctypes.c_int
x11.XInternAtom.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_int]
x11.XInternAtom.restype = ctypes.c_ulong
x11.XGetSelectionOwner.argtypes = [ctypes.c_void_p, ctypes.c_ulong]
x11.XGetSelectionOwner.restype = ctypes.c_ulong
x11.XCloseDisplay.argtypes = [ctypes.c_void_p]

display = x11.XOpenDisplay(None)
if not display:
    raise SystemExit(1)
screen = x11.XDefaultScreen(display)
atom = x11.XInternAtom(display, f"_NET_WM_CM_S{screen}".encode(), 0)
owner = x11.XGetSelectionOwner(display, atom)
x11.XCloseDisplay(display)
raise SystemExit(0 if owner else 1)
PY
    then
      return 0
    fi
    if ! kill -0 "${XCOMPMGR_PID}" >/dev/null 2>&1; then
      cat "${WORK}/xcompmgr.log" >&2 || true
      echo "smoke_m17_m18_linux: xcompmgr exited before claiming the compositor selection" >&2
      exit 1
    fi
    sleep 0.25
  done
  cat "${WORK}/xcompmgr.log" >&2 || true
  echo "smoke_m17_m18_linux: X11 compositor selection did not become ready" >&2
  exit 1
}

wait_for_x11_background() {
  expected="$1"
  for _ in $(seq 1 40); do
    if [ -f "${WORK}/x11-background.ready" ] \
      && [ "$(cat "${WORK}/x11-background.ready")" = "${expected}" ]; then
      sleep 0.20
      return 0
    fi
    if ! kill -0 "${X11_BACKGROUND_PID}" >/dev/null 2>&1; then
      cat "${WORK}/x11-background.log" >&2 || true
      echo "smoke_m17_m18_linux: X11 background client exited early" >&2
      exit 1
    fi
    sleep 0.10
  done
  cat "${WORK}/x11-background.log" >&2 || true
  echo "smoke_m17_m18_linux: X11 background did not acknowledge ${expected}" >&2
  exit 1
}

start_x11_background() {
  color="$1"
  printf '%s\n' "${color}" >"${WORK}/x11-background.command"
  rm -f "${WORK}/x11-background.ready"
  python3 "${ROOT}/script/x11_smoke_background.py" \
    --color "${color}" \
    --command "${WORK}/x11-background.command" \
    --ready "${WORK}/x11-background.ready" \
    >"${WORK}/x11-background.log" 2>&1 &
  X11_BACKGROUND_PID="$!"
  wait_for_x11_background "${color}"
}

set_x11_background() {
  color="$1"
  temporary="${WORK}/x11-background.command.$$"
  printf '%s\n' "${color}" >"${temporary}"
  mv "${temporary}" "${WORK}/x11-background.command"
  kill -USR1 "${X11_BACKGROUND_PID}"
  wait_for_x11_background "${color}"
}

wait_for_frame() {
  frame="$1"
  for _ in $(seq 1 100); do
    if [ -s "${frame}" ]; then
      if cp "${frame}" "${frame}.copy" 2>/dev/null && python3 - "${frame}.copy" <<'PY'
import struct
import sys
import zlib

path = sys.argv[1]
data = open(path, "rb").read()
if not data.startswith(b"\x89PNG\r\n\x1a\n"):
    raise SystemExit(2)

pos = 8
width = height = color = None
idat = bytearray()
while pos + 8 <= len(data):
    length = struct.unpack(">I", data[pos:pos + 4])[0]
    kind = data[pos + 4:pos + 8]
    payload = data[pos + 8:pos + 8 + length]
    pos += 12 + length
    if kind == b"IHDR":
        width, height, bit_depth, color, _comp, _filter, _interlace = struct.unpack(">IIBBBBB", payload)
        if bit_depth != 8 or color != 6:
            raise SystemExit(3)
    elif kind == b"IDAT":
        idat.extend(payload)
    elif kind == b"IEND":
        break

raw = zlib.decompress(bytes(idat))
bpp = 4
stride = width * bpp
prev = bytearray(stride)
opaque = 0
idx = 0
for _y in range(height):
    f = raw[idx]
    idx += 1
    row = bytearray(raw[idx:idx + stride])
    idx += stride
    for i, value in enumerate(row):
        left = row[i - bpp] if i >= bpp else 0
        up = prev[i]
        up_left = prev[i - bpp] if i >= bpp else 0
        if f == 1:
            row[i] = (value + left) & 0xff
        elif f == 2:
            row[i] = (value + up) & 0xff
        elif f == 3:
            row[i] = (value + ((left + up) // 2)) & 0xff
        elif f == 4:
            p = left + up - up_left
            pa = abs(p - left)
            pb = abs(p - up)
            pc = abs(p - up_left)
            row[i] = (value + (left if pa <= pb and pa <= pc else up if pb <= pc else up_left)) & 0xff
        elif f != 0:
            raise SystemExit(4)
    opaque += sum(1 for alpha in row[3::4] if alpha)
    prev = row

if opaque < 50:
    raise SystemExit(5)
print(f"visible alpha pixels: {opaque}")
PY
      then
        return 0
      fi
    fi
    sleep 0.25
  done
  echo "smoke_m17_m18_linux: no visible smoke frame at ${frame}" >&2
  exit 1
}

capture_x11_background_pairs() {
  for _ in $(seq 1 24); do
    pause_runtime
    set_x11_background "#203040"
    import -window root "PNG32:${WORK}/x11-dark.png" >/dev/null 2>&1 || true
    set_x11_background "#d8e6f4"
    import -window root "PNG32:${WORK}/x11-light.png" >/dev/null 2>&1 || true
    set +e
    python3 "${ANALYZER}" \
      --pair x11 "${WORK}/x11-dark.png" "${WORK}/x11-light.png" \
      --require-goose-each \
      --output "${WORK}/x11-analysis.json" >"${WORK}/x11-analysis.log" 2>&1
    analysis_status=$?
    set -e
    resume_runtime
    if [ "${analysis_status}" -eq 0 ]; then
      cat "${WORK}/x11-analysis.log"
      return 0
    fi
    sleep 0.25
  done
  cat "${WORK}/x11-analysis.log" >&2 || true
  echo "smoke_m17_m18_linux: paired X11 compositor capture did not validate" >&2
  exit 1
}

validate_x11_capture_baseline() {
  set_x11_background "#203040"
  import -window root "PNG32:${WORK}/x11-baseline-dark.png"
  set_x11_background "#d8e6f4"
  import -window root "PNG32:${WORK}/x11-baseline-light.png"
  python3 "${ANALYZER}" \
    --pair x11-baseline "${WORK}/x11-baseline-dark.png" "${WORK}/x11-baseline-light.png" \
    --require-no-goose \
    --output "${WORK}/x11-baseline-analysis.json" \
    >"${WORK}/x11-baseline-analysis.log" 2>&1 || {
      cat "${WORK}/x11-baseline-analysis.log" >&2 || true
      echo "smoke_m17_m18_linux: X11 compositor capture baseline is invalid before launch" >&2
      exit 1
    }
  cat "${WORK}/x11-baseline-analysis.log"
}

capture_wayland_background_pairs() {
  for _ in $(seq 1 24); do
    pause_runtime
    set_wayland_background "#203040"
    grim -o "${WAYLAND_FIRST_OUTPUT}" "${WORK}/wayland-first-dark.png" >/dev/null 2>&1 || true
    grim -o "${WAYLAND_SECOND_OUTPUT}" "${WORK}/wayland-second-dark.png" >/dev/null 2>&1 || true
    set_wayland_background "#d8e6f4"
    grim -o "${WAYLAND_FIRST_OUTPUT}" "${WORK}/wayland-first-light.png" >/dev/null 2>&1 || true
    grim -o "${WAYLAND_SECOND_OUTPUT}" "${WORK}/wayland-second-light.png" >/dev/null 2>&1 || true
    set +e
    python3 "${ANALYZER}" \
      --pair "${WAYLAND_FIRST_OUTPUT}" "${WORK}/wayland-first-dark.png" "${WORK}/wayland-first-light.png" \
      --pair "${WAYLAND_SECOND_OUTPUT}" "${WORK}/wayland-second-dark.png" "${WORK}/wayland-second-light.png" \
      --require-goose-any \
      --output "${WORK}/wayland-analysis.json" >"${WORK}/wayland-analysis.log" 2>&1
    analysis_status=$?
    set -e
    resume_runtime
    if [ "${analysis_status}" -eq 0 ]; then
      cat "${WORK}/wayland-analysis.log"
      return 0
    fi
    sleep 0.25
  done
  cat "${WORK}/wayland-analysis.log" >&2 || true
  cat "${WORK}/sway.log" >&2 || true
  echo "smoke_m17_m18_linux: paired per-output Wayland compositor captures did not validate" >&2
  exit 1
}

set_wayland_background() {
  color="$1"
  # Use exact output rules. An inherited system Sway config can leave an existing wallpaper/swaybg
  # surface alive on one output after a later wildcard update. This smoke owns its compositor, so
  # each output must receive the same explicit background before capture.
  swaymsg output "${WAYLAND_FIRST_OUTPUT}" bg "${color}" solid_color >/dev/null
  swaymsg output "${WAYLAND_SECOND_OUTPUT}" bg "${color}" solid_color >/dev/null
  sleep 0.20
}

validate_wayland_capture_baseline() {
  set_wayland_background "#203040"
  grim -o "${WAYLAND_FIRST_OUTPUT}" "${WORK}/wayland-baseline-first-dark.png"
  grim -o "${WAYLAND_SECOND_OUTPUT}" "${WORK}/wayland-baseline-second-dark.png"
  set_wayland_background "#d8e6f4"
  grim -o "${WAYLAND_FIRST_OUTPUT}" "${WORK}/wayland-baseline-first-light.png"
  grim -o "${WAYLAND_SECOND_OUTPUT}" "${WORK}/wayland-baseline-second-light.png"
  python3 "${ANALYZER}" \
    --pair "${WAYLAND_FIRST_OUTPUT}" "${WORK}/wayland-baseline-first-dark.png" "${WORK}/wayland-baseline-first-light.png" \
    --pair "${WAYLAND_SECOND_OUTPUT}" "${WORK}/wayland-baseline-second-dark.png" "${WORK}/wayland-baseline-second-light.png" \
    --require-no-goose \
    --output "${WORK}/wayland-baseline-analysis.json" \
    >"${WORK}/wayland-baseline-analysis.log" 2>&1 || {
      cat "${WORK}/wayland-baseline-analysis.log" >&2 || true
      echo "smoke_m17_m18_linux: Wayland compositor capture baseline is invalid before launch" >&2
      exit 1
    }
  cat "${WORK}/wayland-baseline-analysis.log"
  set_wayland_background "#203040"
}

start_x11_server() {
  need_cmd Xvfb
  need_cmd import
  need_cmd xsetroot
  need_cmd xcompmgr
  export DISPLAY="${HONK300_XVFB_DISPLAY:-:99}"
  Xvfb "${DISPLAY}" -screen 0 1280x720x24 >"${WORK}/xvfb.log" 2>&1 &
  XVFB_PID="$!"
  for _ in $(seq 1 40); do
    if xdpyinfo >/dev/null 2>&1; then
      break
    fi
    sleep 0.25
  done
  xsetroot -solid "#203040"
  if command -v openbox >/dev/null 2>&1; then
    openbox >"${WORK}/openbox.log" 2>&1 &
    OPENBOX_PID="$!"
  fi
  # Simple client-side compositing is xcompmgr's default production-like mode and
  # paints the final desktop into the Composite overlay. Automatic server-side
  # mode (-a) is a debugging mode whose root capture can flatten ARGB windows
  # against black instead of proving their per-pixel composition.
  xcompmgr -n >"${WORK}/xcompmgr.log" 2>&1 &
  XCOMPMGR_PID="$!"
  wait_for_x11_compositor
  start_x11_background "#203040"
  validate_x11_capture_baseline
}

start_sway_headless() {
  need_cmd sway
  need_cmd swaybg
  need_cmd swaymsg
  need_cmd grim
  # AF_UNIX paths are limited to 108 bytes on Linux. Candidate evidence paths include the full
  # target triple and can push sway's generated wayland-N / sway-ipc socket names past that
  # limit, so runtime sockets always live in a short owner-only temporary directory. Logs and
  # screenshots remain in WORK/EVIDENCE_DIR.
  WAYLAND_RUNTIME_DIR="$(mktemp -d /tmp/honk300-wl.XXXXXX)"
  export XDG_RUNTIME_DIR="${WAYLAND_RUNTIME_DIR}"
  chmod 700 "${XDG_RUNTIME_DIR}"
  export WAYLAND_DISPLAY="${HONK300_WAYLAND_DISPLAY:-honk300-wayland-smoke}"
  # Never inherit the runner's distro wallpaper, bar, includes, or output-specific rules. The
  # native smoke compositor has one purpose: expose controlled pixels beneath Honk300's own
  # transparent layer surfaces.
  printf '%s\n' 'output * bg #203040 solid_color' >"${WORK}/sway-smoke.conf"
  chmod 600 "${WORK}/sway-smoke.conf"
  WLR_BACKENDS=headless WLR_LIBINPUT_NO_DEVICES=1 \
    sway -c "${WORK}/sway-smoke.conf" -d >"${WORK}/sway.log" 2>&1 &
  SWAY_PID="$!"
  for _ in $(seq 1 100); do
    if [ -S "${XDG_RUNTIME_DIR}/${WAYLAND_DISPLAY}" ]; then
      break
    fi
    discovered_socket=""
    for socket in "${XDG_RUNTIME_DIR}"/wayland-*; do
      if [ -S "${socket}" ]; then
        WAYLAND_DISPLAY="$(basename "${socket}")"
        export WAYLAND_DISPLAY
        echo "smoke_m17_m18_linux: using Wayland display ${WAYLAND_DISPLAY}"
        discovered_socket=1
        break
      fi
    done
    if [ -n "${discovered_socket}" ]; then
      break
    fi
    sleep 0.25
  done
  if [ ! -S "${XDG_RUNTIME_DIR}/${WAYLAND_DISPLAY}" ]; then
    cat "${WORK}/sway.log" >&2 || true
    echo "smoke_m17_m18_linux: sway headless socket did not appear" >&2
    exit 1
  fi

  for _ in $(seq 1 100); do
    for socket in "${XDG_RUNTIME_DIR}"/sway-ipc.*.sock; do
      if [ -S "${socket}" ]; then
        SWAYSOCK="${socket}"
        export SWAYSOCK
        break 2
      fi
    done
    sleep 0.25
  done
  if [ -z "${SWAYSOCK:-}" ]; then
    cat "${WORK}/sway.log" >&2 || true
    echo "smoke_m17_m18_linux: sway IPC socket did not appear" >&2
    exit 1
  fi

  swaymsg create_output >/dev/null
  for _ in $(seq 1 40); do
    output_names="$(swaymsg -r -t get_outputs | python3 -c 'import json, sys; print(" ".join(output["name"] for output in json.load(sys.stdin) if output.get("active")))')"
    # Virtual headless output names cannot contain spaces.
    # shellcheck disable=SC2086
    set -- ${output_names}
    if [ "$#" -ge 2 ]; then
      break
    fi
    sleep 0.25
  done
  if [ "$#" -lt 2 ]; then
    swaymsg -r -t get_outputs >&2 || true
    echo "smoke_m17_m18_linux: expected at least two active headless outputs" >&2
    exit 1
  fi
  first_output="$1"
  second_output="$2"
  WAYLAND_FIRST_OUTPUT="${first_output}"
  WAYLAND_SECOND_OUTPUT="${second_output}"
  swaymsg output "${first_output}" scale 1.5 pos 0 0 >/dev/null
  swaymsg output "${second_output}" scale 2 pos 1280 0 >/dev/null
  swaymsg -r -t get_outputs | python3 -c '
import json
import sys

active = [output for output in json.load(sys.stdin) if output.get("active")]
scales = sorted(round(float(output.get("scale", 0)), 2) for output in active)
if len(active) < 2 or 1.5 not in scales or 2.0 not in scales:
    raise SystemExit(f"unexpected headless output topology: {active!r}")
print(f"Wayland headless outputs: {len(active)}; scales: {scales}")
'
  validate_wayland_capture_baseline
}

exercise_mode() {
  label="$1"
  frame="$2"
  log="$3"
  shift 3

  echo "smoke_m17_m18_linux: starting ${label}"
  HONK300_SMOKE_FRAME="${frame}" "${BIN}" start --config "${CONFIG}" "$@" >"${log}" 2>&1 &
  PID="$!"
  wait_for_status "${log}"
  cat "${STATUS}"
  grep -q "platform: Linux" "${STATUS}"
  wait_for_frame "${frame}"

  "${BIN}" do honk
  "${BIN}" do mud
  "${BIN}" do wander
  "${BIN}" reload
}

if [ "${BUILD_BIN}" -eq 1 ]; then
  echo "smoke_m17_m18_linux: building debug binary"
  cargo build --manifest-path "${ROOT}/Cargo.toml"
else
  echo "smoke_m17_m18_linux: testing exact HONK300_BIN=${BIN} without rebuilding"
fi
if [ ! -x "${BIN}" ]; then
  echo "smoke_m17_m18_linux: binary is not executable: ${BIN}" >&2
  exit 1
fi
if [ ! -f "${ANALYZER}" ]; then
  echo "smoke_m17_m18_linux: capture analyzer is missing: ${ANALYZER}" >&2
  exit 1
fi
if [ ! -f "${ROOT}/script/x11_smoke_background.py" ]; then
  echo "smoke_m17_m18_linux: X11 background helper is missing" >&2
  exit 1
fi
{
  echo "binary=${BIN}"
  echo "sha256=$(file_sha256 "${BIN}")"
  echo "uname=$(uname -a)"
} >"${WORK}/binary-identity.txt"

echo "smoke_m17_m18_linux: preparing config"
"${BIN}" setup --config "${CONFIG}"

need_cmd python3
need_cmd xdpyinfo

start_x11_server
exercise_mode "X11 visible overlay" "${WORK}/x11-frame.png" "${WORK}/x11-runtime.log"
grep -q "overlay mode is X11" "${WORK}/x11-runtime.log"
grep -q "cursor: supported" "${STATUS}"
grep -q "window: supported" "${STATUS}"
capture_x11_background_pairs
"${BIN}" do nab >"${WORK}/x11-nab.txt" 2>&1 || {
  cat "${WORK}/x11-nab.txt" >&2
  exit 1
}
"${BIN}" stop
wait "${PID}" 2>/dev/null || true
PID=""

unset DISPLAY
start_sway_headless
exercise_mode "Wayland reduced mode" "${WORK}/wayland-frame.png" "${WORK}/wayland-runtime.log" --wayland
grep -q "overlay mode is Wayland" "${WORK}/wayland-runtime.log"
grep -Eq "cursor: (unsupported|failed)" "${STATUS}"
grep -Eq "window: (unsupported|failed)" "${STATUS}"
grep -Eq "collect: (unsupported|failed)" "${STATUS}"
capture_wayland_background_pairs
if "${BIN}" do nab >"${NAB}" 2>&1; then
  echo "smoke_m17_m18_linux: nab unexpectedly succeeded in Wayland reduced mode" >&2
  cat "${NAB}" >&2
  exit 1
fi
grep -q "UNSUPPORTED" "${NAB}"
"${BIN}" stop
wait "${PID}" 2>/dev/null || true
PID=""

echo "passed" >"${WORK}/result.txt"
echo "smoke_m17_m18_linux: compositor-visible X11 and reduced Wayland smoke passed"
