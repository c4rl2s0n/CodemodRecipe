# Completeness rule

After merging derived values, keybinding overrides, and `defaultsTo`, any
**required** arg that is still empty opens the runner instead of applying
(`auto` / `run`). Optional args never block execute.
