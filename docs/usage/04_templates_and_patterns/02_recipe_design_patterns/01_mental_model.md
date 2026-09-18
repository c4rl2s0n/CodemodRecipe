# Mental model

Think in three layers:

1. **Atomic recipes** — one coherent change (create one file, add one field, remove one member).
2. **Orchestrator recipes** — compose atomics into a feature-level workflow.
3. **Shared assets** — templates, maps, and variables reused across recipes.

```mermaid
flowchart TD
  subgraph orchestrator [Orchestrator recipes]
    scaffold["scaffold_*"]
  end
  subgraph create [Create recipes]
    createRepo["create_*"]
  end
  subgraph modify [Modify recipes]
    patch["patch_* / add_* / remove_*"]
  end
  scaffold --> createRepo
  scaffold --> patch
  createRepo --> templates[".codemod/templates/"]
  patch --> maps[".codemod/maps/"]
  patch --> vars[".codemod/variables/"]
```

**One recipe = one coherent change**, composed upward for features. Host API:
[host-api.md](../../02_writing_recipes/02_host_api/00_index.md).
