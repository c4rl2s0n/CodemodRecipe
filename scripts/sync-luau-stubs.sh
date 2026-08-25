#!/usr/bin/env bash
# Sync vscode_extension/stubs/codemod-context.d.luau into export + local .codemod.
# Keep the stub aligned with rust/crates/host/src/lua_runtime.rs `build_ctx`.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/vscode_extension/stubs/codemod-context.d.luau"
mkdir -p "$ROOT/export/.codemod/stubs" "$ROOT/.codemod/stubs"
cp "$SRC" "$ROOT/export/.codemod/stubs/codemod-context.d.luau"
cp "$SRC" "$ROOT/.codemod/stubs/codemod-context.d.luau"
echo "Synced codemod-context.d.luau → export/.codemod/stubs + .codemod/stubs"
