#!/usr/bin/env sh
set -eu

if [ "$(uname -s)" != "Linux" ]; then
  echo "smoke_m17_m18_linux: must run on Linux" >&2
  exit 1
fi

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
BIN="${ROOT}/target/debug/honk300"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/honk300-linux-smoke.XXXXXX")"
CONFIG="${WORK}/config.toml"
STATUS="${WORK}/status.txt"
NAB="${WORK}/nab.txt"
PID=""
XVFB_PID=""
OPENBOX_PID=""
XCOMPMGR_PID=""
SWAY_PID=""

cleanup() {
  "${BIN}" stop >/dev/null 2>&1 || true
  if [ -n "${PID}" ]; then
    wait "${PID}" 2>/dev/null || true
  fi
  if [ -n "${OPENBOX_PID}" ]; then
    kill "${OPENBOX_PID}" >/dev/null 2>&1 || true
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
  rm -rf "${WORK}"
}
trap cleanup EXIT INT TERM

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "smoke_m17_m18_linux: missing required command: $1" >&2
    exit 1
  fi
}

wait_for_status() {
  ready=0
  for _ in $(seq 1 100); do
    if "${BIN}" status >"${STATUS}" 2>&1 && grep -q "honk300: running" "${STATUS}"; then
      ready=1
      break
    fi
    sleep 0.25
  done
  if [ "${ready}" -ne 1 ]; then
    cat "${STATUS}" >&2 || true
    echo "smoke_m17_m18_linux: runtime did not answer status" >&2
    exit 1
  fi
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

wait_for_x11_screenshot() {
  shot="$1"
  for _ in $(seq 1 40); do
    if import -window root "PNG32:${shot}" >/dev/null 2>&1 && python3 - "${shot}" <<'PY'
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
idx = 0
background = 0
goose = 0
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
    for r, g, b, a in zip(row[0::4], row[1::4], row[2::4], row[3::4]):
        if a and abs(r - 0x20) <= 2 and abs(g - 0x30) <= 2 and abs(b - 0x40) <= 2:
            background += 1
        if a and ((r > 215 and g > 215 and b > 215) or (r > 180 and 70 < g < 180 and b < 80)):
            goose += 1
    prev = row

print(f"x11 screenshot background pixels: {background}; goose-like pixels: {goose}")
if goose < 50:
    raise SystemExit(6)
PY
    then
      return 0
    fi
    sleep 0.25
  done
  echo "smoke_m17_m18_linux: no valid X11 root screenshot at ${shot}" >&2
  exit 1
}

start_x11_server() {
  need_cmd Xvfb
  need_cmd import
  need_cmd xsetroot
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
  if command -v xcompmgr >/dev/null 2>&1; then
    xcompmgr -a >"${WORK}/xcompmgr.log" 2>&1 &
    XCOMPMGR_PID="$!"
  fi
}

start_sway_headless() {
  need_cmd sway
  export XDG_RUNTIME_DIR="${WORK}/runtime"
  mkdir -p "${XDG_RUNTIME_DIR}"
  chmod 700 "${XDG_RUNTIME_DIR}"
  export WAYLAND_DISPLAY="${HONK300_WAYLAND_DISPLAY:-honk300-wayland-smoke}"
  WLR_BACKENDS=headless WLR_LIBINPUT_NO_DEVICES=1 sway -d >"${WORK}/sway.log" 2>&1 &
  SWAY_PID="$!"
  for _ in $(seq 1 100); do
    if [ -S "${XDG_RUNTIME_DIR}/${WAYLAND_DISPLAY}" ]; then
      return 0
    fi
    sleep 0.25
  done
  cat "${WORK}/sway.log" >&2 || true
  echo "smoke_m17_m18_linux: sway headless socket did not appear" >&2
  exit 1
}

exercise_mode() {
  label="$1"
  frame="$2"
  log="$3"
  shift 3

  echo "smoke_m17_m18_linux: starting ${label}"
  HONK300_SMOKE_FRAME="${frame}" "${BIN}" start --config "${CONFIG}" "$@" >"${log}" 2>&1 &
  PID="$!"
  wait_for_status
  cat "${STATUS}"
  grep -q "platform: Linux" "${STATUS}"
  wait_for_frame "${frame}"

  "${BIN}" do honk
  "${BIN}" do mud
  "${BIN}" do wander
  "${BIN}" reload
}

echo "smoke_m17_m18_linux: building debug binary"
cargo build --manifest-path "${ROOT}/Cargo.toml"

echo "smoke_m17_m18_linux: preparing config"
"${BIN}" setup --config "${CONFIG}"

need_cmd python3
need_cmd xdpyinfo

start_x11_server
exercise_mode "X11 visible overlay" "${WORK}/x11-frame.png" "${WORK}/x11-runtime.log"
grep -q "overlay mode is X11" "${WORK}/x11-runtime.log"
grep -q "cursor: supported" "${STATUS}"
grep -q "window: supported" "${STATUS}"
wait_for_x11_screenshot "${WORK}/x11-root.png"
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
if "${BIN}" do nab >"${NAB}" 2>&1; then
  echo "smoke_m17_m18_linux: nab unexpectedly succeeded in Wayland reduced mode" >&2
  cat "${NAB}" >&2
  exit 1
fi
grep -q "UNSUPPORTED" "${NAB}"
"${BIN}" stop
wait "${PID}" 2>/dev/null || true
PID=""

echo "smoke_m17_m18_linux: visible X11 and reduced Wayland smoke passed"
