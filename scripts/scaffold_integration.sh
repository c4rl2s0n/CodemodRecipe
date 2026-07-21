#!/usr/bin/env bash
# Run the scaffold mini-project integration test against codemod_host.
# Copies fixtures to a temp workspace; does not modify test/fixtures/scaffold_project/.
#
# Usage:
#   ./scripts/scaffold_integration.sh
#   ./scripts/scaffold_integration.sh --keep
#   ./scripts/scaffold_integration.sh --workspace=/tmp/my_scaffold_test
#
# Faster runs (after building once):
#   CODEMOD_HOST_BIN=rust/target/release/codemod_host ./scripts/scaffold_integration.sh

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
exec node "$ROOT/scripts/scaffold_integration.mjs" "$@"
