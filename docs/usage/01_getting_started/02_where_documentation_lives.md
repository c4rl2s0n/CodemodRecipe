# Where documentation lives

| Layer | Location | Role |
|-------|----------|------|
| **Usage docs** | `docs/usage/` (this guide and siblings) | Product SSOT — bootstrapped beside agent skills |
| **Agent skills** | `.agents/skills/codemod-recipe/` | Thin Architect / Builder indexes into usage docs |
| **Rules** | `.cursor/rules/` | Always-on orientation (not a manual) |
| **Develop docs** | `docs/develop/` | Framework contributors only — not bootstrapped |

After `bootstrap_project`, usage docs are installed under
`.agents/skills/codemod-recipe/docs/`. Skills link into those pages; they are not a
second encyclopedia. See the [docs index](../../README.md).
