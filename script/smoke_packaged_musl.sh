#!/bin/sh
# Exact shipped musl payload, native CPU, private Alpine desktop on disposable CI.
set -eu
test "${GITHUB_ACTIONS:-}" = true
test "$(id -u)" = 0
test -x "$HONK300_BIN"
case "$HONK300_EVIDENCE_DIR" in /work/target/linux-portable-qualification-*/compositor) ;; *) exit 1 ;; esac
apk add --no-cache dbus grim gtk4.0 imagemagick openbox python3 sway swaybg \
  xsetroot xdpyinfo xrandr picom xvfb mesa-dri-gallium font-dejavu libcap-utils
mkdir -p "$HONK300_EVIDENCE_DIR"
apk info -v > "$HONK300_EVIDENCE_DIR/alpine-packages.txt"
compositor="$(command -v sway)"
capabilities="$(getcap "$compositor")"
printf '%s\n' "$capabilities" > "$HONK300_EVIDENCE_DIR/container-file-capabilities.txt"
# Alpine's Sway requests realtime capabilities. This software-rendered private
# desktop does not need them, and Docker correctly refuses that privileged exec.
if test -n "$capabilities"; then setcap -r "$compositor"; fi
adduser -D -u "$HONK_HOST_UID" honkprobe
chown -R honkprobe "$HONK300_EVIDENCE_DIR"
# Sway must run as the ordinary desktop user, with no extra container privileges.
su honkprobe -s /bin/sh -c 'sh /work/script/smoke_m17_m18_linux.sh'
