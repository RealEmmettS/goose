#!/bin/sh
# Run only on a disposable native runner or Alpine container.
set -eu
[ "${GITHUB_ACTIONS:-}" = true ] && [ "$(id -u)" != 0 ]
assets=$1
tag=$2
root=$(mktemp -d)
export XDG_DATA_HOME="$root/Goose data" XDG_CONFIG_HOME="$root/config"
export SHELL=/bin/sh
unset HONK300_LIBC HONK300_TEST_LOCAL_DIR
curl -fsSL --retry 4 https://github.com/RealEmmettS/goose/releases/download/v1.11.3/honk300-installer.sh -o "$root/old.sh"
echo "3ee2f2ec6afa6db0ab7b20ad60b4c7d846c70e799be898b476c0ecb8929b230c  $root/old.sh" | sha256sum -c -
if [ "${VERIFY_PUBLIC_CLI:-}" = true ]; then
  (
    export HOME="$root/cli-home" XDG_DATA_HOME="$root/cli-data" XDG_CONFIG_HOME="$root/cli-config"
    mkdir -p "$HOME"
    sh "$root/old.sh"
    "$HOME/.local/bin/goose" update
    "$HOME/.local/bin/goose" --version | grep -F "${tag#v}"
  )
fi
sh "$root/old.sh"
app="$XDG_DATA_HOME/honk300"
bin="$HOME/.local/bin/goose"
old=$(readlink "$app/install/current")
mkdir -p "$app/media/Notes"
printf 'user media\n' > "$app/media/Notes/my-note.txt"
printf '# retain comments\n[lifecycle]\nautostart_on_login = false\n' > "$app/config.toml"
cp "$app/config.toml" "$root/config.before"
export HONK300_TEST_LOCAL_DIR="$assets"
if HONK300_TEST_FAIL_AFTER_SWAP=1 sh "$assets/honk300-installer.sh"; then exit 1; fi
[ "$(readlink "$app/install/current")" = "$old" ]
for pass in 1 2; do
  sh "$assets/honk300-installer.sh"
  [ "$(readlink "$app/install/current")" != "$old" ]
  python3 - "$app" "$tag" <<'PY'
import json, pathlib, sys
app = pathlib.Path(sys.argv[1])
receipt = json.loads((app / "install-receipt.json").read_text())
assert receipt["tag"] == sys.argv[2] and receipt["origin"] == "shell"
assert (app / "install/current").resolve() == pathlib.Path(receipt["active_release"])
PY
  for alias in honk300 honk goose; do "$HOME/.local/bin/$alias" --version | grep -F "${tag#v}"; done
  "$app/install/current/bin/honk300-settings" --version 2>&1 | grep -F "${tag#v}"
  cmp "$app/config.toml" "$root/config.before"
  grep -Fx 'user media' "$app/media/Notes/my-note.txt"
done
if [ "${VERIFY_PUBLIC_CLI:-}" = true ]; then
  "$bin" install
  "$bin" update
fi
if command -v dpkg >/dev/null; then
  arch=$(dpkg --print-architecture)
  sudo dpkg --install "$assets/honk300-$arch.deb"
  [ "$(readlink "$bin")" = /usr/bin/goose ]
  sh "$assets/honk300-installer.sh"
  sudo dpkg --install "$assets/honk300-$arch.deb"
  "$bin" --version | grep -F "${tag#v}"
  cmp "$app/config.toml" "$root/config.before"
  grep -Fx 'user media' "$app/media/Notes/my-note.txt"
  sudo dpkg --remove honk300
  [ ! -L "$bin" ]
fi
