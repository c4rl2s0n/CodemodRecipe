# Root scan vs child-only

| | In `stack.components` | At `componentScan` | Via `c:component` |
|--|----------------------|--------------------|-------------------|
| **Root** (entity, store, …) | Yes | identify → parse → verify | Yes |
| **Child-only** (property, column, …) | No | Skipped | Yes — parent passes locator |

All component modules are loaded into the registry on reload. The stack only gates **who runs `identify` at scan time**.
