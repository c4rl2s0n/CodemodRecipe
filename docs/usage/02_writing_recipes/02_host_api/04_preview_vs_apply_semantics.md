# Preview vs apply semantics

| Phase | Behavior |
|-------|----------|
| Preview | Runs `apply()` against WorkingTree; skips postExec; builds diff + transcript |
| Apply | Re-runs `apply()` from original file hashes; flushes; runs postExec |
| Token | `previewToken` = hash(recipe + args + starting file contents) |

When preview skipped postExec, apply output may differ from preview patches — the VS Code
review panel surfaces **Files** and **Transcript** views and an `applyMayDiverge` disclaimer.
