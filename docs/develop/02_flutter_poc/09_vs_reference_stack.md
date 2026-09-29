# vs reference_stack

| | `reference_stack` | `flutter_poc` |
|--|-------------------|---------------|
| Purpose | Runtime smoke test | Contract reference (tree + context) |
| Identify | `c:matchFiles` locators | `c:matchFiles("lib/models/{name}.dart")` |
| Parse | Regex | Tree-sitter (`flutter.*` query lib) |
| Structure | Flat `fields` | `children[]` with `source` spans |
| Verify | Graph-aware (`entity.missing_store` via `incoming` edges) | Cross-file model ↔ store via `stores` relation between roots |

See [component-contract.md](../../usage/05_components_and_stacks/03_component_contract/00_index.md).
