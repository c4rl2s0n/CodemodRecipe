import * as vscode from 'vscode';
import type { ExtensionConfig } from '../config/extensionConfig';
import type { HostBridge } from '../host/hostBridge';
import type { RecipeRepository } from '../recipes/recipeRepository';
import type { RecipeRunnerViewProvider } from '../views/recipeRunnerViewProvider';
import type {
  FilePreview,
  RecipeSchema,
  SelectionPayload,
} from '../../shared';
import {
  mergeArgLayers,
  missingRequiredArgNames,
  prefillArgs,
  recipeNeedsHostDerive,
  resolveEditorContext,
} from './recipeContext';

export type InvokeMode = 'auto' | 'run' | 'open';

export interface InvokeArgs {
  recipeId?: string;
  mode?: InvokeMode;
  args?: Record<string, string>;
  /** When set, used instead of active-editor context for `from` prefills. */
  contextValues?: Record<string, string>;
  source?: string;
  languageId?: string;
  filePath?: string;
  cursorOffset?: number;
  selectionStart?: number;
  selectionEnd?: number;
}

export interface InvokeSlotArgs {
  slot?: string;
  mode?: InvokeMode;
  args?: Record<string, string>;
}

export async function invokeRecipe(
  deps: {
    repository: RecipeRepository;
    bridge: HostBridge;
    config: ExtensionConfig;
    runner: RecipeRunnerViewProvider;
  },
  invoke: InvokeArgs
): Promise<void> {
  const recipeId = invoke.recipeId?.trim();
  if (!recipeId) {
    vscode.window.showWarningMessage(
      'Codemod Recipe: invoke requires a recipeId.'
    );
    return;
  }

  const recipe = deps.repository.findById(recipeId);
  if (!recipe) {
    vscode.window.showWarningMessage(
      `Codemod Recipe: unknown recipe id "${recipeId}".`
    );
    return;
  }

  const mode: InvokeMode = invoke.mode ?? 'auto';
  const overrides = invoke.args ?? {};
  const editorContext = resolveEditorContext(deps.config.workspaceRoot);
  const useUriContext = invoke.contextValues != null;
  const contextValues = useUriContext
    ? { ...invoke.contextValues }
    : { ...editorContext.values };
  const source = useUriContext
    ? (invoke.source ?? '')
    : (invoke.source ?? editorContext.source);
  const languageId = useUriContext
    ? (invoke.languageId ?? '')
    : (invoke.languageId ?? editorContext.languageId);
  const filePath = useUriContext
    ? (invoke.filePath ?? '')
    : (invoke.filePath ?? editorContext.filePath);
  const cursorOffset = invoke.cursorOffset ?? (useUriContext ? 0 : editorContext.cursorOffset);
  const selectionStart =
    invoke.selectionStart ?? (useUriContext ? 0 : editorContext.selectionStart);
  const selectionEnd =
    invoke.selectionEnd ?? (useUriContext ? 0 : editorContext.selectionEnd);

  let derived = prefillArgs(recipe, contextValues);
  if (recipeNeedsHostDerive(recipe) && source) {
    try {
      await deps.bridge.ensureHost();
      const response = await deps.bridge.deriveArgs({
        recipe: recipe.id,
        source,
        language: languageId || undefined,
        path: filePath || undefined,
        cursorOffset,
        selectionStart,
        selectionEnd,
        context: contextValues,
      });
      if (response.ok && response.args) {
        derived = { ...derived, ...response.args };
      }
    } catch {
      // Treat host derive failure as unset query args → open runner.
    }
  }

  const filled = mergeArgLayers(derived, overrides);
  const missing = missingRequiredArgNames(recipe, filled);

  if (mode === 'open') {
    deps.runner.run(recipe, filled);
    return;
  }

  if (missing.length > 0) {
    if (mode === 'run') {
      vscode.window.showWarningMessage(
        `Codemod Recipe: missing required args (${missing.join(', ')}); opening runner.`
      );
    }
    deps.runner.run(recipe, filled);
    return;
  }

  // auto / run with complete args → execute
  await executeRecipe(deps, recipe, filled);
}

export async function invokeSlot(
  deps: {
    repository: RecipeRepository;
    bridge: HostBridge;
    config: ExtensionConfig;
    runner: RecipeRunnerViewProvider;
  },
  slotArgs: InvokeSlotArgs
): Promise<void> {
  const slot = normalizeSlotId(slotArgs.slot);
  if (!slot) {
    vscode.window.showWarningMessage(
      'Codemod Recipe: invokeSlot requires a slot id (any character key).'
    );
    return;
  }
  const recipeId = deps.config.slots[slot];
  if (!recipeId) {
    vscode.window.showWarningMessage(
      `Codemod Recipe: no recipe assigned to slot \`${slot}\`.`
    );
    return;
  }
  await invokeRecipe(deps, {
    recipeId,
    mode: slotArgs.mode ?? 'auto',
    args: slotArgs.args,
  });
}

export function normalizeSlotId(raw: string | undefined): string | undefined {
  if (raw == null) {
    return undefined;
  }
  const trimmed = raw.trim();
  if (!trimmed) {
    return undefined;
  }
  // Prefer a single key token; allow multi-char for flexibility (e.g. "f1").
  return trimmed;
}

async function executeRecipe(
  deps: {
    bridge: HostBridge;
    config: ExtensionConfig;
    runner: RecipeRunnerViewProvider;
  },
  recipe: RecipeSchema,
  args: Record<string, string>
): Promise<void> {
  if (deps.config.shortcutConfirmApply) {
    const choice = await vscode.window.showInformationMessage(
      `Apply recipe "${recipe.name}" with derived arguments?`,
      'Apply',
      'Open runner',
      'Cancel'
    );
    if (choice === 'Open runner') {
      deps.runner.run(recipe, args);
      return;
    }
    if (choice !== 'Apply') {
      return;
    }
  }

  try {
    await deps.bridge.ensureHost();
    const preview = await deps.bridge.preview(
      recipe.id,
      args,
      deps.config.previewSnippetLines
    );
    if (!preview.ok || !preview.previewToken) {
      vscode.window.showErrorMessage(
        `Codemod Recipe: preview failed — ${preview.error ?? 'unknown error'}`
      );
      deps.runner.run(recipe, args);
      return;
    }

    const files = preview.files ?? [];
    if (files.length === 0) {
      vscode.window.showInformationMessage(
        `Codemod Recipe: "${recipe.name}" produced no changes.`
      );
      return;
    }

    const selection = selectionIncludingAll(files);
    const applied = await deps.bridge.apply(
      recipe.id,
      args,
      selection,
      preview.previewToken
    );
    if (!applied.ok) {
      vscode.window.showErrorMessage(
        `Codemod Recipe: apply failed — ${applied.error ?? 'unknown error'}`
      );
      deps.runner.run(recipe, args);
      return;
    }

    const count = applied.applied?.length ?? 0;
    vscode.window.showInformationMessage(
      `Applied ${recipe.name} to ${count} file(s).`
    );
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    vscode.window.showErrorMessage(`Codemod Recipe: ${message}`);
    deps.runner.run(recipe, args);
  }
}

function selectionIncludingAll(files: FilePreview[]): SelectionPayload {
  const selection: SelectionPayload = { files: {} };
  for (const file of files) {
    if (file.skipped) {
      continue;
    }
    const patches =
      file.patches.length > 0
        ? file.patches.map((p) => p.index)
        : undefined;
    selection.files[file.path] = {
      include: true,
      ...(patches ? { patches } : {}),
    };
  }
  return selection;
}

export function formatInvokeKeybindingJson(
  recipeId: string,
  mode: InvokeMode = 'auto'
): string {
  return JSON.stringify(
    {
      key: 'ctrl+shift+i 1',
      command: 'codemodRecipe.invoke',
      args: { recipeId, mode },
    },
    null,
    2
  );
}

export function formatSlotKeybindingJson(
  slot: string,
  mode: InvokeMode = 'auto'
): string {
  const prefix = mode === 'open' ? 'ctrl+shift+t' : 'ctrl+shift+i';
  return JSON.stringify(
    {
      key: `${prefix} ${slot}`,
      command: 'codemodRecipe.invokeSlot',
      args: { slot, mode },
    },
    null,
    2
  );
}
