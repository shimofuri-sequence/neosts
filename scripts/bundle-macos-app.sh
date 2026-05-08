#!/usr/bin/env bash

set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "This script must be run on macOS." >&2
  exit 1
fi

if ! cargo bundle --help >/dev/null 2>&1; then
  cat >&2 <<'EOF'
cargo-bundle is not installed.

Install it with:
  cargo install cargo-bundle
EOF
  exit 1
fi

icon_source="assets/NeoSTS_logo.png"
bundle_icon="assets/NeoSTS_logo_512.png"

if [[ ! -f "$icon_source" ]]; then
  echo "App icon source ${icon_source} was not found." >&2
  exit 1
fi

if [[ ! -f "$bundle_icon" || "$icon_source" -nt "$bundle_icon" ]]; then
  if ! command -v sips >/dev/null 2>&1; then
    echo "sips is required to generate ${bundle_icon} from ${icon_source}." >&2
    exit 1
  fi

  sips -z 512 512 "$icon_source" --out "$bundle_icon" >/dev/null
fi

profile="release"
bundle_args=(bundle --format osx --release)

cargo "${bundle_args[@]}" "$@"

app_path="target/${profile}/bundle/osx/NeoSTS.app"

if [[ ! -d "$app_path" ]]; then
  echo "Bundle completed, but ${app_path} was not found." >&2
  exit 1
fi

if [[ -n "${MACOS_CODESIGN_IDENTITY:-}" ]]; then
  codesign --force --deep --options runtime --sign "$MACOS_CODESIGN_IDENTITY" "$app_path"
  codesign --verify --deep --strict --verbose=2 "$app_path"
fi

echo "Created ${app_path}"
