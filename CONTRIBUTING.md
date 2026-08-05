# Contributing to codemod_recipe

Thank you for contributing. This repository is **Rust-first**: the tree-sitter engine,
stdio host, and MCP server live under `rust/`. The VS Code extension is under
`vscode_extension/`.

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) 18+ (extension webview and smoke scripts)
- Git

## Development setup

```bash
git clone https://github.com/c4rl2s0n/CodemodRecipe.git
cd CodemodRecipe
cd rust && cargo build
```

## Running tests

Rust (primary):

```bash
cd rust && cargo test --all
cd rust && cargo clippy --all-targets -- -D warnings
```

Extension webview unit tests:

```bash
cd vscode_extension/src/webview && npm ci && npm test
```

Host protocol smoke (from repo root, after building `codemod_host`):

```bash
cd vscode_extension && ./build.sh
node vscode_extension/scripts/smoke.mjs
```

Scaffold integration (optional):

```bash
./scripts/scaffold_integration.sh
```

## Code style

- Rust: `rustfmt` defaults; run `cargo fmt` before submitting.
- TypeScript/Vue: follow existing patterns under `vscode_extension/src/`.

## Pull requests

1. Keep changes focused; update README, skills under `export/.agents/skills/`, and
   `.cursor/rules/` when behavior or agent workflows change.
2. Add or extend Rust tests for engine/host behavior.
3. Ensure `cargo test --all` passes.

## Agent skills and bootstrap

Bootstrap copies files from `export/` into consumer projects. When editing skills or
rules, update both `.agents/skills/` (workspace copy) and `export/.agents/skills/`
(source of truth for bootstrap).

Human product docs (`docs/getting-started.md`, `docs/writing-recipes.md`,
`docs/README.md`, and generated `docs/generated/dsl-vocabulary.md`) are
**in-repo only** and are not copied by bootstrap. Keep them in sync when
user-facing concepts change; see skill `codemod-recipe-human-docs` under
`.cursor/skills/`.
