#!/usr/bin/env bash
#
# build-assets.sh — run the AmberLume asset/shader builder.
#
# Wraps `cargo run -p builder`: compiles every GLSL shader to SPIR-V, processes
# glTF assets, packs the *.alpaca archives, and regenerates the typed resource
# manifest at target/generated/resources.rs.
#
# Usage:
#   ai_tools/build-assets.sh                 # build everything
#   ai_tools/build-assets.sh <pattern>       # build, then grep the manifest for a symbol
#
# See ai_tools/docs/build-assets.md
set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT"

echo ">> cargo run -p builder"
cargo run -p builder

MANIFEST="$ROOT/target/generated/resources.rs"
echo ">> generated manifest: $MANIFEST"

if [ $# -gt 0 ]; then
  echo ">> constants matching: $*"
  grep -niE "$*" "$MANIFEST" || echo "   (no match — re-check the file stem; names are UPPER, non-alnum -> '_')"
else
  echo ">> tip: ai_tools/build-assets.sh <pattern>   finds a generated const (e.g. 'occlusion')"
fi
