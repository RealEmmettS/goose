#!/bin/sh
# Run inside an already isolated Xvfb/D-Bus session on a disposable runner.
set -eu
test "${GITHUB_ACTIONS:-}" = true
test "$#" -eq 2
payload=$1
evidence=$2
mkdir -p "$evidence"
openbox >"$evidence/window-manager.log" 2>&1 &
manager=$!
xcompmgr -n >"$evidence/compositor.log" 2>&1 &
compositor=$!
trap 'kill "$manager" "$compositor" 2>/dev/null || true' EXIT
export GDK_BACKEND=x11
python3 script/smoke_owned_props_linux.py --binary "$payload/honk300-settings" --evidence "$evidence/native"
python3 script/smoke_runtime_props_linux.py --binary "$payload/honk300" --evidence "$evidence/runtime"
