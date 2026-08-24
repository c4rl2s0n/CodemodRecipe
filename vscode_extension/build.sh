#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

REPO_ROOT="$(cd "$ROOT/.." && pwd)"

echo "Building Rust codemod_host..."
mkdir -p "$ROOT/bin"
cargo build --quiet --manifest-path "$REPO_ROOT/rust/Cargo.toml" -p codemod_recipe_host --bin codemod_host --release
cp "$REPO_ROOT/rust/target/release/codemod_host" "$ROOT/bin/codemod_host"
chmod +x "$ROOT/bin/codemod_host"

# Note: For cross-platform builds, run this on each platform or use a build farm.

echo "Installing npm dependencies..."
npm install

echo "Packaging extension..."
npx --yes @vscode/vsce package

VERSION="$(node -p "require('./package.json').version")"
VSIX="$ROOT/codemod-recipe-${VERSION}.vsix"

if [[ ! -f "$VSIX" ]]; then
  echo "error: expected VSIX at $VSIX" >&2
  exit 1
fi

echo "Build successful! VSIX created at: $VSIX"

if [[ -n "${CODIUM:-}" ]] && command -v "$CODIUM" >/dev/null 2>&1; then
  echo "Installing $VSIX into $CODIUM..."
  "$CODIUM" --install-extension "$VSIX" --force
  echo "Done. Reload $CODIUM (Developer: Reload Window) to activate the update."
else
  echo ""
  echo "To install manually:"
  echo "  code --install-extension $VSIX"
  echo "  or"
  echo "  codium --install-extension $VSIX"
fi
