---
name: codebase-memory
description: First step for structural exploration. Prefer codebase-memory-mcp MCP tools over grep/read. Read the shared playbook.
user-invocable: true
---

# Codebase Memory

## When to use

- Orienting to an unfamiliar subsystem
- Finding where a symbol is defined or who calls it
- Understanding architecture, module boundaries, or impact before a refactor

## Instructions

1. Project context hub: read `.vibe.md` at the repo root.
2. **Read and follow** `.cursor/skills/codebase-memory/reference.md` (canonical playbook).
3. Use `codebase-memory-mcp` MCP tools before grep, glob, or manual file reads.
4. Fall back to manual analysis only when MCP is unavailable or you need line-level edit context after graph orientation.

Setup (if MCP unavailable): https://github.com/DeusData/codebase-memory-mcp
