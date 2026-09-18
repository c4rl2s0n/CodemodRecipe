---
name: codemod-overview
description: >-
  Orientation for codemod-recipe — what it is, when to use MCP vs direct edits,
  project layout. Depth lives in usage docs.
---

# Overview

Read [getting-started.md](../docs/01_getting_started/00_index.md) for the product mental model.

## When to use MCP / recipes

- Structural, multi-file, or AST-boundary edits
- Reusable, idempotent transforms

## When not to

- Trivial one-off text edits → edit directly
- Symbol locate only → Builder intel / codebase-memory

## Modes

- Create/modify `.codemod` assets → [Architect](../architect/SKILL.md)
- Understand or apply → [Builder](../builder/SKILL.md)

## Layout

`.codemod/{recipes,components,maps,types,variables,queries,templates}` — directories are conventions; registry classifies by returned table shape.
