---
name: codemod-recipe-human-docs
description: Use when editing or updating human product docs (docs/getting-started.md, docs/README.md), finishing a change that affects day-1 concepts, or asking how human documentation is organized and maintained.
disable-model-invocation: true
---

# Codemod Recipe Human Docs

## When to use

Use this skill when you are:

- Editing [docs/getting-started.md](../../../docs/getting-started.md), [docs/writing-recipes.md](../../../docs/writing-recipes.md), or [docs/README.md](../../../docs/README.md)
- Finishing a change that alters concepts a new user learns on day 1 (setup, YAML surface, templates/Jinja, maps/vars, languages, extension UX, MCP tools/workflow, design-pattern taxonomy, docs layout)
- Asked how human documentation is structured or how to keep it in sync

## Hard rule

If the change alters something Getting Started teaches, **update the guide in the same change**. If you add a new topic page humans should find from the map, also update `docs/README.md`.

Topic depth still belongs in existing `docs/*.md` and agent skills — Getting Started stays a **thin narrative with deep links**. Authoring depth lives in `docs/writing-recipes.md`. Do not invent new DSL in the guide; copy from shipped examples / schemas. Vocabulary prose stays in `ENTRIES` → regenerate `docs/generated/dsl-vocabulary.md`.

## Where to look

| File | Role |
|------|------|
| `docs/getting-started.md` | Canonical human onboarding walkthrough |
| `docs/writing-recipes.md` | Human recipe authoring guide |
| `docs/generated/dsl-vocabulary.md` | Generated field catalog (do not hand-edit) |
| `docs/README.md` | Documentation map / index |
| `.cursor/skills/codemod-recipe-human-docs/reference.md` | Section map + edit playbook (read this before editing) |

## Related

- Finish-work checklist: `.cursor/skills/codemod-recipe-change-checklist/SKILL.md`
- Repo map: `.cursor/skills/codemod-recipe-repo-orientation/SKILL.md`

## Instructions

1. Read [reference.md](reference.md) in this directory.
2. Apply the edit rules there (when to touch Getting Started vs topic docs only).
3. Keep examples accurate against current DSL (MiniJinja; no legacy navigate DSL).
