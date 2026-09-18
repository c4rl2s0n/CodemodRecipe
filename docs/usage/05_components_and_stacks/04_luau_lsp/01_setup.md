# Setup

1. Install [Luau Language Server](https://marketplace.visualstudio.com/items?itemName=JohnnyMorganz.luau-lsp) (`JohnnyMorganz.luau-lsp`).
2. Run **Codemod Recipe: Scaffold Project** (or bootstrap) so `.vscode/settings.json` includes:

```json
{
  "luau-lsp.types.definitionFiles": [
    ".codemod/stubs/codemod-context.d.luau"
  ],
  "luau-lsp.sourcemap.enabled": false
}
```

3. Add `--!strict` at the top of modules you want typechecked.

After host API changes, run **Update Project Scaffold** or `scripts/generate-host-api-surface.sh` to refresh stubs.
