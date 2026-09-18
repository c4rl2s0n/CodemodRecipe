---
name: codemod-recipe-human-docs
description: >-
  Use when editing product docs (docs/usage/), docs/README.md, finishing a change
  that affects day-1 concepts, or asking how documentation is organized.
  Read docs/develop/01_documentation_system/00_index.md first.
---

# Codemod Recipe Human Docs

## When to use

- Editing anything under [`docs/usage/`](../../../docs/usage/) or [`docs/README.md`](../../../docs/README.md)
- Finishing a change that alters day-1 concepts
- Asked how documentation / bootstrap packaging is structured

## Hard rule

**Read [docs/develop/01_documentation_system/00_index.md](../../../docs/develop/01_documentation_system/00_index.md)** and follow it.

- API/behavior → `docs/usage/` chapters only
- Skills → `.agents/skills/codemod-recipe/**/SKILL.md` (no `export/.agents/`)
- After chapter renames: `python3 scripts/generate_mdbook_summary.py` and `python3 scripts/check_doc_links.py`
- Never reintroduce fat skill `reference.md` or `export/.agents/` duplicates

## Instructions

1. Read [docs/develop/01_documentation_system/00_index.md](../../../docs/develop/01_documentation_system/00_index.md)
2. Apply the checklist there
3. Keep examples accurate (tests, host handlers, or shipped `.luau`)
