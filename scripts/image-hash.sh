#!/usr/bin/env bash
# The image tag (task 007): `h-` and 16 hex characters of a SHA-256 over what
# shapes the image, so the same sources and the same siblings always give the
# same tag and an existing image is reused. Computed ONCE per CI run, after
# the two sibling checkouts, and handed to the build, the "exists?" check and
# the rollout.
#
#   scripts/image-hash.sh <ui-lib checkout> <api checkout>
#
# Inputs: the tracked files (git blob ids) of `app`, `Cargo.toml`,
# `Cargo.lock`, `Dockerfile` and `scripts` of this repository (the Dockerfile
# text, so a base image or a tool version, and the smoke test are in the hash),
# and the HEAD commit of ui-lib and of the api. Not `infra/`, `.task/` or the
# docs: they do not change the image.
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "usage: image-hash.sh <ui-lib checkout> <api checkout>" >&2
  exit 2
fi
ui_lib=$1
api=$2
root=$(cd "$(dirname "$0")/.." && pwd)

head_of() { # head_of NAME DIR
  git -C "$2" rev-parse HEAD 2>/dev/null || { echo "image-hash: $1 ($2) is not a git checkout" >&2; return 1; }
}
ui_lib_head=$(head_of ui-lib "$ui_lib")
api_head=$(head_of api "$api")

if command -v sha256sum >/dev/null 2>&1; then
  sha() { sha256sum; }
else
  sha() { shasum -a 256; }
fi

hash=$(
  {
    git -C "$root" ls-files -s app Cargo.toml Cargo.lock Dockerfile scripts
    echo "ui-lib $ui_lib_head"
    echo "api $api_head"
  } | sha | cut -c1-16
)
echo "h-$hash"
