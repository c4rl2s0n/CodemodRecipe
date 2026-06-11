#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

CODIUM="${CODIUM:-codium}"
if ! command -v "$CODIUM" >/dev/null 2>&1; then
  echo "error: codium not found (set CODIUM to the CLI path to override)" >&2
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

echo "Installing $VSIX into Codium..."
"$CODIUM" --install-extension "$VSIX" --force

echo "Done. Reload Codium (Developer: Reload Window) to activate the update."
