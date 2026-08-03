---
name: codemod-recipe-change-checklist
description: Use when finishing a change to ensure docs/skills/rules stay in sync with the updated behavior. Covers READMEs, human Getting Started docs, modular skills, and Cursor rules/config.
disable-model-invocation: true
---

# Cross-cutting Change Checklist

## When to use

Use this skill before concluding work if you:
- changed the JSON-over-stdio protocol / message contract
- changed recipe schema serialization or arg input metadata
- changed `model.rs` / `dsl::` / `ENTRIES` / DSL codegen outputs (`generated-dsl-surface.json`, schemas) — never hand-edit generated schema shape
- changed the VS Code language toolkit (`vscode_extension/src/extension/language/`)
- changed preview/diff/apply selection semantics
- changed UI behavior that affects how agents should implement future Vue changes

## What to update (docs, skills, rules)

If your change affects behavior, conventions, configuration, or how developers should work with the repo, update:

1. **Project READMEs**
   - Root: `README.md`
   - Extension: `vscode_extension/README.md`

2. **Human product docs**
   - `docs/getting-started.md` and `docs/README.md` when the end-user mental model or day-1 workflows change (setup, YAML surface, templates/Jinja, maps/vars, languages, extension UX, MCP workflow, design-pattern taxonomy, docs layout)
   - How to edit: skill `codemod-recipe-human-docs` (read its `reference.md`)
   - Deep reference still belongs in topic `docs/*.md`; Getting Started stays a thin narrative with deep links

3. **Modular skills library**
   - `.cursor/skills/*/SKILL.md` for any subsystem you changed
   - `.cursor/skills/codebase-memory/reference.md` when subsystems or exploration entry points change
   - If you created/changed a new subsystem or workflow, add a new skill file rather than only editing existing ones.

4. **Cursor rules / rules-like guidance (if present)**
   - If your repo uses `.cursor/rules/*` (or similar guidance files), update those when conventions or workflows changed.
   - If the repo does not currently have rules, include this checklist as the primary “where to document changes” guidance.

5. **Vibe agent docs**
   - `.vibe.md` Agent tooling section when exploration entry points change
   - `.vibe/prompts/cli.md` when upgrading `mistral-vibe` (re-diff bundled `cli.md`)

## What to update (protocol & types)

If you changed any of these, you almost certainly need coordinated updates across multiple files:

- **Host↔TS contract**
  - Rust host commands / response JSON shapes:
    - `rust/crates/host/src/dispatch.rs`, `protocol.rs`
  - TS request/response types + markers:
    - `vscode_extension/src/shared/messages.ts`
    - `vscode_extension/src/extension/host/hostProtocol.ts`

- **Webview controller behavior**
  - Preview ordering/stale suppression:
    - `vscode_extension/src/webview/src/composables/useRunnerController.ts`

- **Selection semantics**
  - UI selection model:
    - `vscode_extension/src/webview/src/lib/selection.ts`
  - Host apply semantics:
    - `rust/crates/host/src/dispatch.rs` (selection parsing)

## Completion self-check

- [ ] Do the relevant skills’ “Where to look” links still point to correct files?
- [ ] Did I update the skill text where it describes the changed behavior?
- [ ] Did I update one or more READMEs if end-user or developer workflows changed?
- [ ] Did Getting Started stay accurate for anything I changed that a new user would learn?
- [ ] If I introduced a new message/command/type, did I update the checklist’s “what to update” mapping?

