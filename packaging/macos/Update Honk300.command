#!/bin/sh
set -eu

CONTENTS_DIR="$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd -P)"
exec "$CONTENTS_DIR/MacOS/honk300" __control-surface-update
