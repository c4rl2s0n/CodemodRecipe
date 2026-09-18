# Preview vs apply

- Both run `apply(c, args)`. Preview uses a WorkingTree; **postExec is skipped** (listed in transcript).
- Apply **re-runs** Luau and flushes before postExec / `postExecution`.
- `previewToken` = hash(recipe + args + starting file contents).
