#!/usr/bin/env bash
# Full product build: regenerate host API surface, release-build codemod_host,
# then package the VS Code extension.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

echo "Regenerating host API surface..."
"$ROOT/scripts/generate-host-api-surface.sh"

echo "Building Rust codemod_host (release)..."
mkdir -p "$ROOT/vscode_extension/bin"
cargo build --quiet --manifest-path "$ROOT/rust/Cargo.toml" \
  -p codemod_recipe_host --bin codemod_host --release
cp "$ROOT/rust/target/release/codemod_host" "$ROOT/vscode_extension/bin/codemod_host"
chmod +x "$ROOT/vscode_extension/bin/codemod_host"

# Note: For cross-platform builds, run this on each platform or use a build farm.

echo "Packaging VS Code extension..."
"$ROOT/vscode_extension/build.sh"
