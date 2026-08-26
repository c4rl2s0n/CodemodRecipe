#!/usr/bin/env bash
# Package the VS Code / Codium extension (npm + vsce).
# Expects a prebuilt host binary at bin/codemod_host — use the repo-root ./build.sh
# for the full product build (surface gen + Rust release + this script).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

HOST_BIN="$ROOT/bin/codemod_host"
if [[ ! -x "$HOST_BIN" ]]; then
  echo "error: missing executable $HOST_BIN" >&2
  echo "Run ./build.sh from the repository root to build codemod_host and package the extension." >&2
  exit 1
fi

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
