# Validation

On reload, when any components exist:

1. `stack.luau` must be present.
2. Every id in `stack.components` must match a loaded component module.

Recipe-only projects (no components) do not require a stack.
