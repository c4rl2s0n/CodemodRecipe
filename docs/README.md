# Documentation

| Tree | Audience | Bootstrapped? |
|------|----------|---------------|
| **[usage/](usage/00_index.md)** | Product usage | Yes → `.agents/skills/codemod-recipe/docs/` |
| **[develop/](develop/)** | Framework contributors | No |

**Start here:** [usage/00_index.md](usage/00_index.md) (or [usage/01_getting_started/00_index.md](usage/01_getting_started/00_index.md)).

Agent skills (Architect / Builder) live under [`.agents/skills/codemod-recipe/`](../.agents/skills/codemod-recipe/) and index into usage docs. How to edit docs correctly: [develop/01_documentation_system/00_index.md](develop/01_documentation_system/00_index.md).

Human interactive browse (this repo, usage + develop): `cd docs && mdbook-mermaid install . && mdbook serve`. Consumers use the usage-only book under `.agents/skills/codemod-recipe/docs/` (see [usage/00_index.md](usage/00_index.md)).

## Develop

| Doc | Contents |
|-----|----------|
| [develop/01_documentation_system/00_index.md](develop/01_documentation_system/00_index.md) | Doc system, bootstrap manifest, edit rules |
| [develop/02_flutter_poc/00_index.md](develop/02_flutter_poc/00_index.md) | Fixture walkthrough |
| [../ARCHITECTURE.md](../ARCHITECTURE.md) | Host architecture |
| [../CONTRIBUTING.md](../CONTRIBUTING.md) | Build and test |
