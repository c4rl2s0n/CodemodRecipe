---
name: codemod-recipe-change-checklist
description: Use when finishing a change to ensure docs/skills/rules stay in sync with the updated behavior. Provides a cross-cutting “what to update” checklist across README files, the modular skills library, and any Cursor rules/config.
disable-model-invocation: true
---

# Cross-cutting Change Checklist

## When to use

Use this skill before concluding work if you:
- changed the JSON-over-stdio protocol / message contract
- changed recipe schema serialization or arg input metadata
- changed preview/diff/apply selection semantics
- changed UI behavior that affects how agents should implement future Vue changes

## What to update (docs, skills, rules)

If your change affects behavior, conventions, configuration, or how developers should work with the repo, update:

1. **Project READMEs**
   - Root: `README.md`
   - Extension: `vscode_extension/README.md`
   - (and any relevant example README under `example/` when the change affects those workflows)

2. **Modular skills library**
   - `.cursor/skills/*/SKILL.md` for any subsystem you changed
   - If you created/changed a new subsystem or workflow, add a new skill file rather than only editing existing ones.

3. **Cursor rules / rules-like guidance (if present)**
   - If your repo uses `.cursor/rules/*` (or similar guidance files), update those when conventions or workflows changed.
   - If the repo does not currently have rules, include this checklist as the primary “where to document changes” guidance.

## What to update (protocol & types)

If you changed any of these, you almost certainly need coordinated updates across multiple files:

- **Dart↔TS contract**
  - Dart host commands / response JSON shapes:
    - `lib/src/vscode/codemod_host.dart`
  - TS request/response types + markers:
    - `vscode_extension/src/types.ts`
    - `vscode_extension/src/constants.ts`
    - `vscode_extension/src/views/recipeRunnerMessages.ts`
    - `vscode_extension/src/webview/extensionToWebviewMessages.ts`
    - `vscode_extension/webview-ui/src/messages.ts`

- **Webview controller behavior**
  - Preview ordering/stale suppression:
    - `vscode_extension/webview-ui/src/composables/useRunnerController.ts`

- **Selection semantics**
  - UI selection model:
    - `vscode_extension/webview-ui/src/lib/selection.ts`
  - Host PatchSelector apply semantics:
    - `lib/src/vscode/patch_selector.dart`

## Completion self-check

- [ ] Do the relevant skills’ “Where to look” links still point to correct files?
- [ ] Did I update the skill text where it describes the changed behavior?
- [ ] Did I update one or more READMEs if end-user or developer workflows changed?
- [ ] If I introduced a new message/command/type, did I update the checklist’s “what to update” mapping?

