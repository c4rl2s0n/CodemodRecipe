# Human docs — structure and edit playbook

Canonical human onboarding lives in this repository only (not under `export/`, not
copied by `bootstrap_project`).

| File | Purpose |
|------|---------|
| [docs/getting-started.md](../../../docs/getting-started.md) | Narrative day-1 walkthrough |
| [docs/writing-recipes.md](../../../docs/writing-recipes.md) | How to author recipes (capture/anchor, steps, workflow) |
| [docs/generated/dsl-vocabulary.md](../../../docs/generated/dsl-vocabulary.md) | Generated field/enum catalog — **do not hand-edit** |
| [docs/README.md](../../../docs/README.md) | Documentation map / index |

Agent skills (`export/.agents/skills/`) and rulesets (`export/rulesets/`) are for
**consumer** workspaces. Do not put “update Getting Started” obligations there —
those projects do not ship `docs/`.

## Intended section map (`docs/getting-started.md`)

Keep this outline stable. Add or rename sections only when the day-1 mental model
actually changes.

1. **What you’re getting** — product pitch; host + two clients; preview → apply; mermaid data flow
2. **Where documentation lives** — human docs vs agent skills vs rules
3. **Pick a path** — VS Code vs MCP (link out for path-specific setup)
4. **Setup (enough to succeed)** — prerequisites, `.codemod/` layout, extension + MCP checklists
5. **Core concepts** — recipes (incl. capture/anchor gloss), queries, edit ops, Jinja/templates, maps/variables, composition, languages, design patterns (each short + deep link to writing-recipes / topic docs)
6. **Day-1 success** — run via VS Code and via MCP; idempotency
7. **Optional companion** — codebase-memory
8. **What agents use** — skills/rules vs human docs
9. **Next steps** — link table (writing-recipes + vocabulary near the top)

`docs/README.md` groups the same material plus architecture/contributing; lead with
Getting Started → Writing recipes.

## When to update Getting Started

**Update** `docs/getting-started.md` when you change something a new user would learn there, for example:

- Setup steps for extension or MCP
- Recipe YAML shape (step kinds, required fields, composition)
- Template / Jinja surface that appears in the minimal examples
- Maps / variables discovery model
- Language defaults or how `language:` is introduced
- Extension day-1 UX (Recipes / Runner / apply flow)
- MCP tool order or previewToken semantics at the overview level
- Design-pattern prefixes (`create_` / `add_` / `scaffold_` / `remove_`)
- Where documentation lives (layers or bootstrap vs in-repo)

Also update `docs/README.md` when you **add, rename, or remove** a topic doc that belongs in the map.

Update `docs/writing-recipes.md` when the authoring workflow or capture/anchor/op
mental model changes (not every new enum value — those go to `ENTRIES` → regenerate).

## Generated vocabulary

Field and enum **prose** lives only in `rust/crates/yaml/src/dsl_vocabulary.rs`
(`ENTRIES`). Structure comes from `model.rs` / schemars.

After changing `ENTRIES`, `dsl::` wires, or model fields, run
`scripts/generate-dsl-artifacts.sh`. That refreshes JSON Schema, keyword docs,
TextMate, and **`docs/generated/dsl-vocabulary.md`**. Never hand-edit the
generated markdown; CI diffs it.

## When not to update Getting Started

Touch only the **topic doc** / vocabulary / skill when the change is deep reference and the day-1 mental model and minimal examples stay correct, for example:

- An extra Jinja filter or obscure MCP argument
- Internal host protocol fields not mentioned in Getting Started
- Contributor-only Vue/webview implementation details
- New golden fixtures that do not change the product story

If in doubt: update the topic doc for sure; update Getting Started only if a new user reading it would be misled.

## Edit rules

1. **Thin narrative, deep links** — Prefer one short example + a link to `docs/*.md` over pasting filter catalogs or full MCP tool tables.
2. **No invented DSL** — Copy shapes from README, fixtures, or schemas. Canonical templates are MiniJinja; do not revive legacy navigate DSL.
3. **New early concept** — Add a short concept subsection under Core concepts (or Setup if it is setup-only) and a row/link in `docs/README.md`. Prefer linking [writing-recipes.md](../../../docs/writing-recipes.md) for authoring depth.
4. **Rename / remove** — Update Getting Started and the docs index in the **same** change.
5. **Style** — Tutorial voice, scannable headings, tables OK, mermaid OK. Gloss agent-only jargon when it appears.
6. **Cross-links** — Path-specific docs (`new-project-rust-mcp.md`, extension README) should keep a one-liner pointing at Getting Started.

## Finish checklist

- [ ] Getting Started still matches the behavior I changed (or I confirmed deep-reference-only)
- [ ] `docs/README.md` links are current if I added/renamed a topic page
- [ ] Writing recipes / vocabulary still accurate if authoring surface changed
- [ ] Examples still parse against current schema / shipped recipes
- [ ] Cross-cutting checklist skill considered: `.cursor/skills/codemod-recipe-change-checklist/SKILL.md`
