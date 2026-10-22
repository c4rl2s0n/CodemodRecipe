#!/usr/bin/env bash
# D-018: Single generated surface — regenerate into vscode_extension/stubs/ (SSOT),
# then copy to export/ + .codemod/ for bootstrap/IDE. Do not edit the copies by hand.
# Run scripts/generate-host-api-surface.sh to regenerate the stub from Rust first.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/vscode_extension/stubs/codemod-context.d.luau"
mkdir -p "$ROOT/export/.codemod/stubs" "$ROOT/.codemod/stubs"
cp "$SRC" "$ROOT/export/.codemod/stubs/codemod-context.d.luau"
cp "$SRC" "$ROOT/.codemod/stubs/codemod-context.d.luau"
echo "Synced codemod-context.d.luau → export/.codemod/stubs + .codemod/stubs"
