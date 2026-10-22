# Dogfood (self-hosting)

Applying codemod-recipe to its own repository: the root `.codemod/` holds
**self-hosting assets** (components/queries/stack modeling this repo's Rust
crates, host protocol, VS Code extension wiring, docs), scanned via
`codemod_mcp` (see `.cursor/mcp.json`). Test fixtures live under `test/` —
never in the root `.codemod/`.

| Page | |
|------|---|
| [Issues log](01_issues.md) | Bugs, papercuts, and architectural findings from dogfooding |
| [Coverage](02_coverage.md) | Component inventory, verified findings, backlog |
| [Decisions](03_decisions.md) | Phase 2 calls: stack ops, no junction, deferred graph-aware recipes |
| [Architecture graph](04_architecture_graph.md) | Command-triangle kind + instance mermaid (scan view) |
| [Target architecture](05_target_architecture.md) | Full-system expected layering, crate DAG, smells |
