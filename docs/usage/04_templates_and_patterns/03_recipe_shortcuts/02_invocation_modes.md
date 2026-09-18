# Invocation modes

Commands:

- `codemodRecipe.invoke` — `{ "recipeId": "…", "mode": "auto"|"run"|"open", "args"?: { … } }`
- `codemodRecipe.invokeSlot` — `{ "slot": "b", "mode": "…", "args"?: { … } }`

| Mode | Behavior |
|------|----------|
| `auto` (default) | All required args filled (after derivation + `defaultsTo`) → preview then apply; else open runner prefilled |
| `run` | Same completeness check; if incomplete, show a warning and still open the runner |
| `open` | Always open the runner prefilled |

Optional `args` in the keybinding override derived values (binding wins).

**Execute** means: host preview → apply all non-skipped changes → toast with file count.
On preview/apply failure, the runner opens with the derived args (never silent fail).

Setting `codemodRecipe.shortcutConfirmApply` (default `false`) asks before auto-apply.

Discovery without a fixed recipe:

- **Codemod Recipe: Run From Cursor Context** (`Ctrl+Alt+R`) QuickPicks recipes
  that match any derived arg, then uses `auto` mode.
- The **Recipes** sidebar has a **Context** expander listing the same Tier A
  matches (local `from` / `contextKey` prefills). Use **Run** / **Open**, or
  right-click **Create shortcut…** to assign a slot (optionally with fixed args).
  Slot badges (`slot:b`) appear on matching recipes; chord text is a hint for
  the default `Ctrl+Shift+I` / `Ctrl+Shift+T` prefixes.
