#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT/rust"
cargo run -q -p codemod_recipe_yaml --bin codemod_dsl_codegen
