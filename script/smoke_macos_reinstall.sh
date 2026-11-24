#!/bin/bash
# Exercise the signed graphical helper without dismissing its final modal or launching Goose.
set -euo pipefail
assets=$1
version=${2#v}
test "${GITHUB_ACTIONS:-}" = true
root=$(mktemp -d "${RUNNER_TEMP}/goose-reinstall.XXXXXX")
runner_home=$HOME
export HOME="$root/home"
mount="$root/mount"
mkdir -p "$HOME" "$mount"
helper_pid=
cleanup() {
  if [[ -n "$helper_pid" ]]; then kill "$helper_pid" 2>/dev/null || true; wait "$helper_pid" || true; fi
  hdiutil detach "$mount" >/dev/null 2>&1 || true
}
trap cleanup EXIT
xcodebuild -version
if [[ "${CHECK_XCODE_27:-}" = true ]]; then
  xcodebuild -version | grep -Fx 'Build version 27A266a'
  curl -fsSL --retry 4 https://github.com/RealEmmettS/goose/releases/download/v1.11.3/honk300-universal2.app.zip -o "$root/old.zip"
  echo "6d692454dae6ca6abd414961e83158911498ddbbd47ae849d13daeef7549e81c  $root/old.zip" | shasum -a 256 -c -
  ditto -x -k "$root/old.zip" "$root/old"
  old="$root/old/Honk300.app/Contents/MacOS/honk300"
  if /usr/bin/lipo "$old" -verify_arch x86_64 arm64 2>"$root/baseline.log"; then
    echo 'Baseline unexpectedly passed on the pinned toolchain' >&2; exit 1
  fi
  cat "$root/baseline.log"
  grep -F 'exactly one input file' "$root/baseline.log"
  for arch in arm64 x86_64; do /usr/bin/lipo "$old" -verify_arch "$arch"; done
fi
curl -fsSL --retry 4 https://github.com/RealEmmettS/goose/releases/download/v1.11.3/honk300-installer.sh -o "$root/old.sh"
echo "3ee2f2ec6afa6db0ab7b20ad60b4c7d846c70e799be898b476c0ecb8929b230c  $root/old.sh" | shasum -a 256 -c -
if [[ "${CHECK_XCODE_27:-}" = true ]]; then
  # Model an installation performed before the user upgraded Xcode. Split only the two
  # known lipo checks in the pinned *fixture*. All candidate and public installers run as shipped.
  python3 - "$root/old.sh" <<'PY'
from pathlib import Path
import sys
path = Path(sys.argv[1])
text = path.read_text()
for variable in ("STAGED_SETTINGS", "STAGED_BINARY"):
    check = 'lipo "$' + variable + '" -verify_arch '
    assert text.count(check + "x86_64 arm64") == 1
    text = text.replace(check + "x86_64 arm64", check + "x86_64\n  " + check + "arm64")
path.write_text(text)
PY
fi
if [[ "${VERIFY_PUBLIC_CLI:-}" = true ]]; then
  (
    export HOME="$runner_home"
    test ! -e "$HOME/Applications/Honk300.app"
    sh "$root/old.sh"
    "$HOME/.local/bin/goose" update --check --json
    "$HOME/Applications/Honk300.app/Contents/MacOS/honk300" update --check --json
    HONK300_TEST_LOCAL_DIR="$assets" sh "$assets/honk300-installer.sh"
    "$HOME/.local/bin/goose" update --check --json
    "$HOME/Applications/Honk300.app/Contents/MacOS/honk300" update --check --json
  )
  (
    export HOME="$root/cli-home"
    mkdir -p "$HOME"
    sh "$root/old.sh"
    "$HOME/.local/bin/goose" --version | grep -F 1.11.3
    cli="$HOME/.local/bin/goose"
    bundle="$HOME/Applications/Honk300.app/Contents/MacOS/honk300"
    python3 - "$HOME" <<'PY'
import json, pathlib, sys
home = pathlib.Path(sys.argv[1])
receipt = json.loads((home / "Library/Application Support/honk300/install-receipt.json").read_bytes())
print({k: receipt[k] for k in ("origin","installer_family","scope","install_root","active_release","artifact")}, flush=True)
for path in (home, home / ".local/bin/goose", home / "Applications/Honk300.app/Contents/MacOS/honk300"):
    print(str(path), str(path.resolve(strict=True)), flush=True)
PY
    "$cli" update --check --json
    "$bundle" update --check --json
    "$cli" update
    "$HOME/.local/bin/goose" --version | grep -F "$version"
  )
fi
sh "$root/old.sh"
"$HOME/.local/bin/goose" --version | grep -F 1.11.3
hdiutil attach -readonly -nobrowse -mountpoint "$mount" "$assets/honk300-universal2.dmg"
app="$HOME/Applications/Honk300.app"
receipt="$HOME/Library/Application Support/honk300/install-receipt.json"
agent="$HOME/Library/LaunchAgents/dev.emmetts.honk300.plist"
payload="$mount/Honk300.app/Contents/MacOS/honk300"
for arch in x86_64 arm64; do
  /usr/bin/lipo "$payload" -thin "$arch" -output "$root/$arch"
  other=x86_64; [[ "$arch" = x86_64 ]] && other=arm64
  /usr/bin/lipo "$root/$arch" -verify_arch "$arch"
  if /usr/bin/lipo "$root/$arch" -verify_arch "$other"; then exit 1; fi
done
if /usr/bin/lipo "$0" -verify_arch arm64; then exit 1; fi
for startup in off on; do
  if [[ "$startup" = on ]]; then
    "$payload" install --autostart
    test -f "$agent"
    HONK300_TEST_LOCAL_DIR="$assets" sh "$assets/honk300-installer.sh"
  fi
  python3 -c 'import json,sys; assert json.load(open(sys.argv[1]))["origin"] == "shell"' "$receipt"
  "$mount/Install Honk300.app/Contents/MacOS/Install Honk300" >"$root/helper.log" 2>&1 &
  helper_pid=$!
  for ((i=0; i<120; i++)); do
    if python3 -c 'import json,sys; r=json.load(open(sys.argv[1])); sys.exit(r["origin"]!="mac-app" or r["version"]!=sys.argv[2])' "$receipt" 2>/dev/null; then break; fi
    kill -0 "$helper_pid" || { cat "$root/helper.log"; exit 1; }
    sleep 0.5
  done
  python3 - "$receipt" "$version" "$startup" <<'PY'
import json, sys
r = json.load(open(sys.argv[1]))
assert (r["origin"], r["version"], r["autostart"]["enabled"]) == ("mac-app", sys.argv[2], sys.argv[3] == "on")
PY
  codesign --verify --deep --strict "$app"
  cmp "$payload" "$app/Contents/MacOS/honk300"
  cmp "$mount/Honk300.app/Contents/MacOS/honk300-settings" "$app/Contents/MacOS/honk300-settings"
  if [[ "$startup" = on ]]; then test -f "$agent"; else test ! -e "$agent"; fi
  kill "$helper_pid"; wait "$helper_pid" || true; helper_pid=
done
if [[ "${VERIFY_PUBLIC_CLI:-}" = true ]]; then
  "$HOME/.local/bin/goose" install
  "$HOME/.local/bin/goose" update
  python3 -c 'import json,sys; r=json.load(open(sys.argv[1])); assert r["origin"]=="mac-app" and r["version"]==sys.argv[2]' "$receipt" "$version"
  test -f "$agent"
fi
