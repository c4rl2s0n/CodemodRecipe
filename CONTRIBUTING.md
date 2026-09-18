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

Host protocol smoke (from repo root, after `./build.sh`):

```bash
./build.sh
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

1. Keep changes focused; update README, skills under `.agents/skills/codemod-recipe/`,
   and `export/rulesets/` when behavior or agent workflows change. See
   [docs/develop/documentation.md](docs/develop/documentation.md).
2. Add or extend Rust tests for engine/host behavior.
3. Ensure `cargo test --all` passes.

## Agent skills and bootstrap

Skills SSOT: **`.agents/skills/codemod-recipe/`** (Architect / Builder). There is no
`export/.agents/` copy.

[`export/bootstrap.toml`](export/bootstrap.toml) tells the host build which trees to
embed (skills, `docs/usage/`, rulesets, scaffold). Usage docs land in consumers at
`.agents/skills/codemod-recipe/docs/`.

When editing product docs, edit **`docs/usage/`** only. In this repo,
`.agents/skills/codemod-recipe/docs` is a symlink to `docs/usage`.

Framework-contributor docs live under `docs/develop/` and are **not** bootstrapped.
