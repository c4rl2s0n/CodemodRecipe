# Bootstrap

[`export/bootstrap.toml`](../../../export/bootstrap.toml) lists embed sources. [`rust/crates/host/build.rs`](../../../rust/crates/host/build.rs) reads it at compile time.

| Source | Dest in consumer |
|--------|------------------|
| `.agents/skills/codemod-recipe/` (exclude `docs`) | same path |
| `docs/usage/` | `.agents/skills/codemod-recipe/docs/` |
| `export/rulesets/`, `export/companions/` | pack-prefixed, filtered at install |
| `export/.codemod/`, `export/.vscode/` | `.codemod/`, `.vscode/` |

In this repo, `.agents/skills/codemod-recipe/docs` is a **symlink** to `docs/usage` so skill relative links match consumers. Always edit `docs/usage/`, never treat the symlink as a second SSOT.
