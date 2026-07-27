---
name: yaml-dsl
description: Reference for the YAML DSL used by the Rust host. Covers edit/create/delete/recipe steps, tree-sitter query ops, templates, maps, and validation.
user-invocable: false
allowed-tools: ["read", "grep"]
---

# YAML DSL Reference (Rust Host)

> **Bootstrap source of truth:** `export/.agents/skills/codemod-yaml-dsl/` (SKILL.md + reference.md).

Rust implementation:

- `rust/crates/yaml/src/model.rs`
- `rust/crates/yaml/src/validate.rs`
- `rust/crates/yaml/src/compose.rs`
- `rust/crates/host/src/runner.rs`

Recipes require `id` and `steps`. There is no separate DSL version field.

See the export skill `codemod-yaml-dsl` for full syntax, templates, guards, and composition.
