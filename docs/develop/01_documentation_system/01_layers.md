# Layers

| Layer | Location | Bootstrapped? | Role |
|-------|----------|---------------|------|
| **Usage docs** | [`docs/usage/`](../../usage/) (numbered chapters) | Yes → `.agents/skills/codemod-recipe/docs/` | Product SSOT |
| **Usage skills** | [`.agents/skills/codemod-recipe/`](../../../.agents/skills/codemod-recipe/) | Yes | Thin Architect / Builder indexes |
| **Rules / scaffold** | [`export/rulesets/`](../../../export/rulesets/), `.codemod`, `.vscode` | Yes | Policy + empty dirs |
| **Develop docs / skills** | [`docs/develop/`](../), [`.cursor/skills/`](../../../.cursor/skills/) | **No** | Framework contributors |

**There is no `export/.agents/` copy.** Skills are edited only under `.agents/skills/codemod-recipe/`.
