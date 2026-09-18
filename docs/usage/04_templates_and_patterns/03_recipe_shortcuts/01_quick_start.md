# Quick start

1. Declare `from` on recipe args so required fields resolve from the open file / cursor.
2. Map a slot (or bind `codemodRecipe.invoke` directly).
3. Press the chord — complete args apply; incomplete ones open the runner.

## Workspace slots

```jsonc
// .vscode/settings.json
{
  "codemodRecipe.slots": {
    "1": "flutter.add_cubit",
    "b": "flutter.add_bloc",
    "c": {
      "recipeId": "flutter.add_cubit",
      "mode": "auto",
      "args": { "kind": "widget" }
    }
  }
}
```

Slot values may be a **recipe id string** or a structured object with optional
`mode` and fixed `args` (merged over derived values at invoke time).

Slot ids are **any character key** VS Code can use as a chord second stroke
(`1`, `b`, `[`, …) — not only digits.

Default sample chords (when the extension is installed):

| Chord | Command | Mode |
|-------|---------|------|
| `Ctrl+Shift+I` then `1` | `codemodRecipe.invokeSlot` | `auto` |
| `Ctrl+Shift+T` then `1` | `codemodRecipe.invokeSlot` | `open` |

Add letter chords yourself in Keyboard Shortcuts, for example:

```json
{
  "key": "ctrl+shift+i b",
  "command": "codemodRecipe.invokeSlot",
  "args": { "slot": "b", "mode": "auto" }
}
```

Mac uses `cmd` instead of `ctrl` in the shipped defaults.

## Authoring helpers

On a recipe Luau **top-level** `id = "..."` line (CodeLens):

- **Copy invoke keybinding** — clipboard JSON for `codemodRecipe.invoke`
- **Assign to slot…** — writes `codemodRecipe.slots` and optionally copies a slot keybinding
- **Open in Recipe Runner** — opens the webview prefilled

`c:run("child.id")` string literals do **not** get these lenses.

Commands also appear in the Command Palette:
`Codemod Recipe: Copy Invoke Keybinding`, `Assign to Slot`, `Copy Slot Keybinding`.
