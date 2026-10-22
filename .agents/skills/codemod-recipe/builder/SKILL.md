---
name: codemod-recipe-builder
description: >-
  Use codemod-recipe against a workspace: understand structure (intel) or apply
  registered recipes/components (apply). Indexes usage docs and MCP checklists.
---

# Builder

**Use** the framework on application code. Prefer registered recipes/components over inventing Luau unless the user asks to author (then switch to [Architect](../architect/SKILL.md)).

## Route

| Need | Open |
|------|------|
| Map instances, findings, AST, or locate edit targets | [intel/SKILL.md](intel/SKILL.md) |
| Preview and apply patches / component hooks | [apply/SKILL.md](apply/SKILL.md) |

## Golden rule

Always dry-run / preview before apply. Pass `previewToken` to `apply_recipe`. For components: `component_dry_run_hook` before `component_apply_hook`.

## Docs

| Doc | Contents |
|-----|----------|
| [docs/06_mcp_and_clients/01_codemod_mcp/00_index.md](../docs/06_mcp_and_clients/01_codemod_mcp/00_index.md) | Full MCP tool reference |
| [docs/05_components_and_stacks/01_components/00_index.md](../docs/05_components_and_stacks/01_components/00_index.md) | Scan / scaffold / operations |
| [docs/01_getting_started/00_index.md](../docs/01_getting_started/00_index.md) | Day-1 product overview |

## Prompt templates

- `Scan the stack and summarize instances + findings from summaries only`
- `Preview recipe X with these args; show files/transcript; wait for my OK before apply`
- `Dry-run component operation addField on instance … then apply if clean`
