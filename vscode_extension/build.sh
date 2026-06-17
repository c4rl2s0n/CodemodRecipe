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
  echo "warning: dart not found in PATH, trying FVM..." >&2
  
  # Try FVM (Flutter Version Management)
  # FVM stores versions in $HOME/fvm/versions/<version>/bin/
  # First, try to use fvm command to get the correct Dart
  if command -v "fvm" >/dev/null 2>&1; then
    # Try to get Dart from FVM using the project's .fvmrc if available
    if [[ -f "$REPO_ROOT/.fvmrc" ]]; then
      DART_CMD="$(cd "$REPO_ROOT" && fvm dart --version 2>/dev/null && fvm which dart 2>/dev/null || echo "")"
      if [[ -n "$DART_CMD" && -x "$DART_CMD" ]]; then
        echo "Using FVM Dart: $DART_CMD" >&2
      else
        DART_CMD="dart"
      fi
    fi
  fi
  
  # Try direct FVM paths
  if ! command -v "$DART_CMD" >/dev/null 2>&1; then
    for fvm_version in stable 3.38.7; do
      if [[ -x "$HOME/fvm/versions/$fvm_version/bin/dart" ]]; then
        DART_CMD="$HOME/fvm/versions/$fvm_version/bin/dart"
        echo "Using FVM Dart: $DART_CMD" >&2
        break
      fi
    done
  fi
  
  # Try common Dart SDK locations
  if ! command -v "$DART_CMD" >/dev/null 2>&1; then
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
  fi
  
  if ! command -v "$DART_CMD" >/dev/null 2>&1; then
    echo "error: dart not found. Please install Dart SDK (https://dart.dev/get-dart) or set up FVM" >&2
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

echo "Build successful! VSIX created at: $VSIX"

# Optional: Install into Codium/VSCode if available
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
