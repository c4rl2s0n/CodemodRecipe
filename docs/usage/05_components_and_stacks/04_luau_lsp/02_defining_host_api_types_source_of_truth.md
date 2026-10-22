# Defining host API types (source of truth)

All Luau-facing host types live in [`rust/crates/lua/src/host_api/specs.rs`](../../../../rust/crates/lua/src/host_api/specs.rs). Do **not** hardcode type shapes in the stub generator.

| Kind | How |
|------|-----|
| **Input specs** (`InsertSpec`, `QueryCaptureSpec`, …) | `JsonSchema` + `LuaSpec` (or manual `HostSpec`) + `#[host_method(spec = …)]` |
| **Return shapes** (`QueryHit`, `ByteSpan`, `MatchFileRow`, …) | `JsonSchema` + `HostType` + `#[host_method(return_type = …)]` / `Array(…)` |
| **Stubs / completions** | Generated from `HOST_API_SURFACE` by `scripts/generate-host-api-surface.sh` |

Optional fields in stubs use Luau syntax `field: T?` (not TypeScript `field?: T`).

## `c:query` vs helpers

- **`QueryCaptureSpec`** — `capture` required → `c:query`
- **`QueryTableSpec`** — `capture` optional → `queryExists` / `queryCount` / `queryMatches` / `expectMatch` / `expectNoMatch`

Query hits are `{ span = { from, to }, text, kind }` (`ByteSpan` uses `from`/`to` because Lua reserves `end`).

## `c:resolve` vs `c:template`

| | `c:resolve` | `c:template` |
|--|-------------|--------------|
| Job | Path string fill-in | Content rendering |
| Placeholders | `{key}` | MiniJinja `{{ key }}` |
| Side effects | None | File load / recipe args, maps, vars |

Prefer `resolve` for workspace-relative paths; use `template` for generated file bodies.
