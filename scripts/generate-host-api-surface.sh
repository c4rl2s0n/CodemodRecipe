#!/usr/bin/env bash
# Regenerate vscode_extension/stubs/codemod-context.d.luau and
# vscode_extension/src/extension/language/hostApiSurface.ts from the Rust host
# API spec structs, then sync the .d.luau stub into export + local .codemod.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export CARGO_TARGET_DIR="$ROOT/rust/target"
cd "$ROOT/rust"
cargo run -p codemod_recipe_host --bin generate_host_api_surface
"$ROOT/scripts/sync-luau-stubs.sh"
