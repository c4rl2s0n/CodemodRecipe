# Root scan vs child-only

| | In `stack.components` | At `componentScan` | Via `c:component` |
|--|----------------------|--------------------|-------------------|
| **Root** (entity, store, …) | Yes | identify → parse, then relate → verify over the whole forest | Yes |
| **Child-only** (property, column, …) | No | No `identify`/`parse` of their own — but post-scan `relate`/`verify` still run for their instances | Yes — parent passes locator |

All component modules are loaded into the registry on reload. The stack only gates **who runs `identify` at scan time**.
