# Human docs — structure and edit playbook

Canonical human onboarding lives in this repository only (not under `export/`, not
copied by `bootstrap_project`).

| File | Purpose |
|------|---------|
| [docs/getting-started.md](../../../docs/getting-started.md) | Narrative day-1 walkthrough |
| [docs/writing-recipes.md](../../../docs/writing-recipes.md) | How to author Luau recipes (capture/anchor, host API) |
| [docs/README.md](../../../docs/README.md) | Documentation map / index |

Agent skills (`export/.agents/skills/`) and rulesets (`export/rulesets/`) are for
**consumer** workspaces. Do not put “update Getting Started” obligations there.

## Intended section map (`docs/getting-started.md`)

Keep this outline stable. Add or rename sections only when the day-1 mental model
actually changes.

1. **What you’re getting** — product pitch; host + two clients; preview → apply; mermaid data flow
2. **Where documentation lives** — human docs vs agent skills vs rules
3. **Pick a path** — VS Code vs MCP (link out for path-specific setup)
4. **Setup (enough to succeed)** — prerequisites, `.codemod/` layout, extension + MCP checklists
5. **Core concepts** — Luau recipes, host API, queries, templates, maps/variables, composition, languages, design patterns
6. **Day-1 success** — run via VS Code and via MCP; idempotency; Files + Transcript review
7. **Optional companion** — codebase-memory
8. **What agents use** — skills/rules vs human docs
9. **Next steps** — link table (writing-recipes near the top)

## When to update Getting Started

Update when you change something a new user would learn there, for example:

- Setup steps for extension or MCP
- Luau recipe module shape or host API surface
- Template / MiniJinja usage via `c:applyTemplate`
- Maps / variables / query lib discovery
- Language defaults or how `language` on ops is introduced
- Extension day-1 UX (Recipes / Runner / Files + Transcript / applyMayDiverge)
- MCP tool order or previewToken semantics
- Design-pattern prefixes

Also update `docs/README.md` when you add, rename, or remove a topic doc.

Update `docs/writing-recipes.md` when the authoring workflow or capture/anchor mental model changes.

## Host API SSOT

Host API detail lives in skill `codemod-luau-recipes`. The generated stub
[`docs/generated/dsl-vocabulary.md`](../../../docs/generated/dsl-vocabulary.md)
points there.

## Edit rules

1. **Thin narrative, deep links** — Prefer one short Luau example + a link to topic docs.
2. **No invented API** — Copy shapes from tests, `lua_runtime.rs`, or shipped `.luau` recipes.
3. **Rename / remove** — Update Getting Started and the docs index in the **same** change.
