#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

# Find the repository root (two levels up from vscode_extension)
REPO_ROOT="$(cd "$ROOT/.." && pwd)"

# Compile Dart host for all platforms
echo "Compiling Dart host executables..."

# Determine which Dart executable to use
DART_CMD="dart"
if ! command -v "$DART_CMD" >/dev/null 2>&1; then
  echo "warning: dart not found in PATH, trying common locations..." >&2
  # Try common Dart SDK locations
  for candidate in \
    /usr/local/bin/dart \
    /opt/dart-sdk/bin/dart \
    "$HOME/.dart/sdk/bin/dart" \
    "$HOME/.pub-cache/bin/dart" \
    /snap/bin/dart
  do
    if [[ -x "$candidate" ]]; then
      DART_CMD="$candidate"
      break
    fi
  done
  
  if ! command -v "$DART_CMD" >/dev/null 2>&1; then
    echo "error: dart not found. Please install Dart SDK (https://dart.dev/get-dart)" >&2
    exit 1
  fi
fi

echo "Using Dart: $DART_CMD"

# Create bin directory if it doesn't exist
mkdir -p "$ROOT/bin"

# Compile for current platform
echo "Compiling codemod_host for current platform..."
"$DART_CMD" compile exe "$REPO_ROOT/bin/codemod_host.dart" -o "$ROOT/bin/codemod_host"

# Note: For cross-platform builds, you would need to run this on each platform
# or use a build farm. For development, compiling on the current platform is sufficient.

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
