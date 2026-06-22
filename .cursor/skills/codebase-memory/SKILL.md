---
name: codebase-memory
description: Use as the first step when exploring this codebase — architecture, symbol lookup, call chains, impact analysis. Prefer codebase-memory-mcp MCP tools over grep/glob/manual file reads. Read the playbook for workflows.
---

# Codebase Memory

## When to use

- Orienting to an unfamiliar subsystem
- Finding where a symbol is defined or who calls it
- Understanding architecture, module boundaries, or impact before a refactor
- Any structural question where grep/glob would scan many files

## Instructions

1. **Read and follow** [reference.md](reference.md) in this directory.
2. Use MCP tools from `codebase-memory-mcp` before manual file exploration.
3. Fall back to grep/read only after graph tools have oriented you, or when MCP is unavailable.
