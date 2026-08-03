---
name: code-quality
description: >-
  Run Clippy, rustfmt, TypeScript checks, and structure/practices review for
  codemod-recipe (Rust workspace + VS Code extension). Use when the user asks
  for code quality, clippy, lint, refactor hygiene, best practices, or a
  quality pass before merge.
---

# Code Quality

Maintainer skill for the **Rust** workspace (`rust/`) and **VS Code** extension
(`vscode_extension/`). Legacy Dart under `lib/` is not the primary runtime.

For crate maps and “where to change X”, see
[codemod-recipe-repo-orientation](../codemod-recipe-repo-orientation/SKILL.md).
For extension language toolkit rules, see
[codemod-recipe-vscode-extension-backend](../codemod-recipe-vscode-extension-backend/SKILL.md).
Full checklists and anti-patterns: [reference.md](reference.md).

## When to use

- User asks to run Clippy, fix lint warnings, or improve code quality
- Pre-merge / PR hygiene on Rust or extension code
- After large refactors that may violate crate or protocol boundaries

## Workflow (run in order)

Copy and track:

```
Quality pass:
- [ ] 1. Scope
- [ ] 2. Automated gates
- [ ] 3. Structure review
- [ ] 4. Practices review
- [ ] 5. Report
```

### 1. Scope

Default: entire Rust workspace + `vscode_extension/src/`.
If the user names files/crates, narrow gates and review to those (still run
workspace Clippy when unsure — CI does).

### 2. Automated gates (must pass)

```bash
# Rust format
cd rust && cargo fmt --all -- --check
# If check fails:
cd rust && cargo fmt --all

# Clippy (same as CI)
cd rust && cargo clippy --all-targets -- -D warnings

# Extension types (no full webview rebuild required)
cd vscode_extension && npx tsc -p ./ --noEmit

# When webview or shared/ changed:
cd vscode_extension && npm run test:webview
```

Fix every Clippy/fmt/`tsc` failure before moving on. Prefer real fixes over
`#[allow(...)]`. Justified allows need a one-line reason (see reference).

### 3. Structure review

Use **codebase-memory** first for ownership and call chains, then spot-check
edit sites.

Verify:

- Crate boundaries (`core` / `yaml` / `engine` / `host`) — see reference table
- DSL vocabulary via `yaml` `dsl::` / `dsl_structure` / `model`, not ad-hoc strings
- Paths via `codemod_recipe_core::resource_path`, not ad-hoc `join` / `..` checks
- After DSL/structure changes: `scripts/generate-dsl-artifacts.sh` and committed
  generated schemas
- Extension: no hardcoded DSL container children; use generated surface + host catalog
- Do not hand-edit `vscode_extension/media/*`
- Host JSON shapes stay in sync: Rust `protocol.rs` / `dispatch.rs` ↔
  `vscode_extension/src/shared/messages.ts`

### 4. Practices review

Apply SOLID / DRY / KISS **to this codebase** (reference § Practices):

- One responsibility per module/function; split I/O + parsing + policy mixes
- Reuse existing owners instead of new util dumps
- Match neighboring style; no speculative abstractions
- Actionable errors; tests for behavioral changes (`cargo test --all` / Vitest)

Fix clear violations in scope. Do not drive-by refactor unrelated modules.

### 5. Report

End with severity-ordered findings:

| Bucket | Meaning |
|--------|---------|
| Fixed | Changed in this pass |
| Must-fix | Still blocking gates or clear bugs |
| Suggestion | Non-blocking improvements |

List commands run and any remaining debt. **Do not commit** unless the user asks.

## After behavior changes

If quality fixes change contracts or workflows, run
[codemod-recipe-change-checklist](../codemod-recipe-change-checklist/SKILL.md).
