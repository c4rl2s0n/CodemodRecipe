# Args conventions

Use a consistent vocabulary across related recipes:

| Arg | `inputKind` | Used by |
|-----|-------------|---------|
| `file` | `file` | Generic edit recipes |
| `feature` | `text` | Scaffold orchestrators (drives paths via `c:template`) |
| `className` | `text` | Class-targeted edits |
| `fieldName` / `eventName` | `text` | Domain names; type resolved via map |
| `fieldType` | optional or map-backed | When not inferable from map |

**Path convention:** pick one style per project and stick to it.

- Scaffolds derive paths from args: `c:template({ inline = "lib/features/{{ feature | snake_case }}/{{ feature | snake_case }}_state.dart" })`
- Modify recipes take explicit `file`, or use the same path template as scaffolds
