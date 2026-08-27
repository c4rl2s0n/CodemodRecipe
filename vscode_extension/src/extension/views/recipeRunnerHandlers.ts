import * as path from 'path';
import * as vscode from 'vscode';
import { COMMANDS, DIFF } from '../constants';
import type { ExtensionConfig } from '../config/extensionConfig';
import type { DiffContentProvider } from '../diff/diffContentProvider';
import type { HostBridge } from '../host/hostBridge';
import {
  EXTENSION_TO_WEBVIEW,
  RUNNER_SESSION_PHASE,
  WEBVIEW_TO_EXTENSION,
  assertNever,
  FILE_PREVIEW_KIND,
  type ArgValues,
  type FilePreview,
  type SelectionPayload,
  type WebviewToExtensionMessage,
} from '../../shared';
import type { RecipeRunnerState } from './recipeRunnerState';
import { createShortcutForRecipe } from '../recipes/createShortcut';
import { invokeRecipe } from '../recipes/recipeInvoke';
import type { RecipeRepository } from '../recipes/recipeRepository';

export interface RecipeRunnerHandlerHost {
  readonly workspaceRoot: string;
  readonly state: RecipeRunnerState;
  readonly bridge: HostBridge;
  readonly config: ExtensionConfig;
  readonly diffProvider: DiffContentProvider;
  get previewInFlight(): boolean;
  setPreviewInFlight(value: boolean): void;
  /** Monotonic generation; bumped when a preview is superseded or finished. */
  get previewGeneration(): number;
  beginPreviewGeneration(): number;
  get scaffoldHandler(): (() => Promise<void>) | undefined;
  postState(): void;
  postMessage(message: unknown): void;
  run(
    recipe: import('../../shared').RecipeSchema,
    initialArgs?: ArgValues
  ): void;
  ensureRecipeDetails(
    recipe: import('../../shared').RecipeSchema
  ): Promise<import('../../shared').RecipeSchema>;
  argsKey(args: ArgValues): string;
  get recipeRepository(): RecipeRepository | undefined;
  /** Reset host preview flag + bump session (webview recreate / ready). */
  resetSession(): void;
  /** Publish runtime diagnostics (Problems panel). */
  publishRuntimeDiagnostics?(diagnostics: readonly import('../../shared').RecipeDiagnostic[]): void;
}

export async function handleWebviewMessage(
  host: RecipeRunnerHandlerHost,
  message: WebviewToExtensionMessage
): Promise<void> {
  switch (message.type) {
    case WEBVIEW_TO_EXTENSION.ready:
      host.resetSession();
      host.postState();
      break;
    case WEBVIEW_TO_EXTENSION.bootstrapRetry:
      await vscode.commands.executeCommand(COMMANDS.bootstrap);
      break;
    case WEBVIEW_TO_EXTENSION.selectRecipe:
      await handleSelectRecipe(host, message.id);
      break;
    case WEBVIEW_TO_EXTENSION.refreshRecipes:
      await vscode.commands.executeCommand(COMMANDS.refresh);
      break;
    case WEBVIEW_TO_EXTENSION.configureHost:
      await vscode.commands.executeCommand(COMMANDS.configureCodemodRoot);
      break;
    case WEBVIEW_TO_EXTENSION.scaffoldProject:
      if (host.scaffoldHandler) {
        await host.scaffoldHandler();
      } else {
        await vscode.commands.executeCommand(COMMANDS.scaffoldProject);
      }
      break;
    case WEBVIEW_TO_EXTENSION.openRecipeFile:
      await handleOpenRecipeFile(host, message.id);
      break;
    case WEBVIEW_TO_EXTENSION.pickFile:
      await handlePickPath(host, message.arg, false, message.requestId);
      break;
    case WEBVIEW_TO_EXTENSION.pickDirectory:
      await handlePickPath(host, message.arg, true, message.requestId);
      break;
    case WEBVIEW_TO_EXTENSION.preview:
      // Do not await — must observe superseding preview messages while apply runs.
      void handlePreview(host, message.args, message.requestId);
      break;
    case WEBVIEW_TO_EXTENSION.openDiff:
      await handleOpenDiffByPath(host, message.path);
      break;
    case WEBVIEW_TO_EXTENSION.apply:
      await handleApply(host, message.selection);
      break;
    case WEBVIEW_TO_EXTENSION.invokeRecipe:
      await handleInvokeRecipe(
        host,
        message.recipeId,
        message.mode,
        message.args
      );
      break;
    case WEBVIEW_TO_EXTENSION.createShortcut:
      await handleCreateShortcut(host, message.recipeId, message.args);
      break;
    default:
      assertNever(message);
  }
}

async function handleInvokeRecipe(
  host: RecipeRunnerHandlerHost,
  recipeId: string,
  mode: 'auto' | 'run' | 'open',
  args?: ArgValues
): Promise<void> {
  const repository = host.recipeRepository;
  if (!repository) {
    return;
  }
  await invokeRecipe(
    {
      repository,
      bridge: host.bridge,
      config: host.config,
      runner: host,
    },
    { recipeId, mode, args }
  );
}

async function handleCreateShortcut(
  host: RecipeRunnerHandlerHost,
  recipeId: string,
  args?: ArgValues
): Promise<void> {
  const repository = host.recipeRepository;
  if (!repository) {
    return;
  }
  const scalar: Record<string, string> = {};
  if (args) {
    for (const [k, v] of Object.entries(args)) {
      if (typeof v === 'string') {
        scalar[k] = v;
      }
    }
  }
  await createShortcutForRecipe(
    { config: host.config, repository },
    recipeId,
    Object.keys(scalar).length > 0 ? scalar : undefined
  );
}

async function handleSelectRecipe(
  host: RecipeRunnerHandlerHost,
  recipeId: string
): Promise<void> {
  const recipe = host.state.recipes.find((item) => item.id === recipeId);
  if (recipe) {
    host.run(recipe);
  }
}

async function handleOpenRecipeFile(
  host: RecipeRunnerHandlerHost,
  recipeId: string
): Promise<void> {
  const recipe = host.state.recipes.find((item) => item.id === recipeId);
  const sourceFile = recipe?.sourceFile;
  if (!sourceFile) {
    vscode.window.showInformationMessage(
      `Codemod Recipe: no source file recorded for "${recipeId}".`
    );
    return;
  }
  const abs = path.isAbsolute(sourceFile)
    ? sourceFile
    : path.join(host.workspaceRoot, sourceFile);
  const uri = vscode.Uri.file(abs);
  try {
    const doc = await vscode.workspace.openTextDocument(uri);
    await vscode.window.showTextDocument(doc, { preview: true });
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    vscode.window.showErrorMessage(
      `Codemod Recipe: failed to open ${sourceFile}: ${message}`
    );
  }
}

async function handlePickPath(
  host: RecipeRunnerHandlerHost,
  arg: string,
  directory: boolean,
  requestId: number
): Promise<void> {
  host.state.setSessionPhase(RUNNER_SESSION_PHASE.pickingPath, requestId);
  host.postState();
  try {
    const picked = await vscode.window.showOpenDialog({
      defaultUri: vscode.Uri.file(host.workspaceRoot),
      canSelectMany: false,
      canSelectFiles: !directory,
      canSelectFolders: directory,
      filters: directory
        ? undefined
        : {
            Source: ['dart', 'rs', 'java', 'kt', 'kts', 'sql', 'ts', 'js', 'py'],
            All: ['*'],
          },
      openLabel: 'Select',
    });
    if (!picked?.[0]) {
      host.postMessage({
        type: EXTENSION_TO_WEBVIEW.filePicked,
        arg,
        requestId,
        cancelled: true,
      });
      return;
    }

    const rel = path.relative(host.workspaceRoot, picked[0].fsPath);
    host.postMessage({
      type: EXTENSION_TO_WEBVIEW.filePicked,
      arg,
      requestId,
      value: rel.startsWith('..') ? picked[0].fsPath : rel,
    });
  } finally {
    if (host.state.sessionPhase === RUNNER_SESSION_PHASE.pickingPath) {
      host.state.setSessionPhase(
        host.state.lastPreviewToken
          ? RUNNER_SESSION_PHASE.reviewing
          : RUNNER_SESSION_PHASE.idle
      );
      host.postState();
    }
  }
}

async function handlePreview(
  host: RecipeRunnerHandlerHost,
  args: ArgValues,
  requestId: number
): Promise<void> {
  const recipe = host.state.currentRecipe;
  if (!recipe) {
    host.postMessage({
      type: EXTENSION_TO_WEBVIEW.error,
      message: 'No recipe selected',
      requestId,
    });
    return;
  }

  if (host.previewInFlight) {
    // Kill in-flight Luau apply and start this request instead.
    host.bridge.cancelInFlight('Preview superseded');
    host.setPreviewInFlight(false);
  }

  const generation = host.beginPreviewGeneration();
  host.setPreviewInFlight(true);
  host.state.setSessionPhase(RUNNER_SESSION_PHASE.previewing, requestId);
  host.postState();
  host.postMessage({
    type: EXTENSION_TO_WEBVIEW.previewState,
    inFlight: true,
    requestId,
  });
  host.state.lastArgs = args;
  const argsKey = host.argsKey(args);

  const isCurrent = () => generation === host.previewGeneration;

  try {
    const response = await host.bridge.preview(
      recipe.id,
      args,
      host.config.previewSnippetLines
    );
    if (!isCurrent()) {
      return;
    }
    if (!response.ok) {
      host.state.setSessionPhase(RUNNER_SESSION_PHASE.idle);
      const diagnostics = runtimeDiagnosticsFromFailure(
        response.error ?? 'Preview failed',
        response.diagnostics,
        recipe
      );
      host.publishRuntimeDiagnostics?.(diagnostics);
      host.postMessage({
        type: EXTENSION_TO_WEBVIEW.error,
        message: response.error ?? 'Preview failed',
        requestId,
        diagnostics,
        transcript: response.transcript ?? [],
      });
      host.postState();
      return;
    }

    host.state.lastFiles = response.files ?? [];
    host.state.lastPreviewToken = response.previewToken;
    host.state.lastPreviewArgsKey = argsKey;
    host.state.setSessionPhase(RUNNER_SESSION_PHASE.reviewing, requestId);
    host.publishRuntimeDiagnostics?.([]);
    host.postMessage({
      type: EXTENSION_TO_WEBVIEW.previewResult,
      files: host.state.lastFiles,
      transcript: response.transcript ?? [],
      applyMayDiverge: response.applyMayDiverge ?? false,
      requestId,
      argsKey,
    });
    host.postState();
  } catch (err) {
    if (!isCurrent()) {
      return;
    }
    const message = err instanceof Error ? err.message : String(err);
    const cancelled =
      message.includes('superseded') || message.includes('cancelled');
    host.state.setSessionPhase(RUNNER_SESSION_PHASE.idle);
    if (!cancelled) {
      const diagnostics = runtimeDiagnosticsFromFailure(message || 'Preview failed', undefined, recipe);
      host.publishRuntimeDiagnostics?.(diagnostics);
      host.postMessage({
        type: EXTENSION_TO_WEBVIEW.error,
        message: message || 'Preview failed',
        requestId,
        diagnostics,
      });
    }
    host.postState();
  } finally {
    if (isCurrent()) {
      host.setPreviewInFlight(false);
      host.postMessage({
        type: EXTENSION_TO_WEBVIEW.previewState,
        inFlight: false,
        requestId,
      });
    }
  }
}

function runtimeDiagnosticsFromFailure(
  message: string,
  fromHost: readonly import('../../shared').RecipeDiagnostic[] | undefined,
  recipe: import('../../shared').RecipeSchema
): import('../../shared').RecipeDiagnostic[] {
  if (fromHost && fromHost.length > 0) {
    return [...fromHost];
  }
  return [
    {
      severity: 'error',
      code: 'E_RECIPE_RUNTIME',
      message,
      sources: recipe.sourceFile
        ? [{ file: recipe.sourceFile }]
        : undefined,
      relatedRecipe: recipe.id,
      hint: 'Fix the recipe apply() body, then re-run preview.',
    },
  ];
}

async function handleApply(
  host: RecipeRunnerHandlerHost,
  selection: SelectionPayload
): Promise<void> {
  const recipe = host.state.currentRecipe;
  if (!recipe) {
    host.postMessage({
      type: EXTENSION_TO_WEBVIEW.error,
      message: 'No recipe selected',
    });
    return;
  }

  const previewToken = host.state.lastPreviewToken;
  if (!previewToken) {
    host.postMessage({
      type: EXTENSION_TO_WEBVIEW.error,
      message: 'Preview is out of date. Re-run preview before applying.',
    });
    return;
  }

  host.state.setSessionPhase(RUNNER_SESSION_PHASE.applying);
  host.postState();
  try {
    const response = await host.bridge.apply(
      recipe.id,
      host.state.lastArgs,
      selection,
      previewToken
    );
    if (!response.ok) {
      const diagnostics = runtimeDiagnosticsFromFailure(
        response.error ?? 'Apply failed',
        response.diagnostics,
        recipe
      );
      host.publishRuntimeDiagnostics?.(diagnostics);
      host.postMessage({
        type: EXTENSION_TO_WEBVIEW.error,
        message: response.error ?? 'Apply failed',
        diagnostics,
      });
      return;
    }

    const count = response.applied?.length ?? 0;
    vscode.window.showInformationMessage(
      `Applied ${recipe.name} to ${count} file(s).`
    );
    host.state.lastPreviewToken = undefined;
    host.state.lastFiles = [];
    host.state.lastPreviewArgsKey = undefined;
    host.publishRuntimeDiagnostics?.([]);
    host.postMessage({
      type: EXTENSION_TO_WEBVIEW.applyResult,
      applied: response.applied ?? [],
    });
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    const diagnostics = runtimeDiagnosticsFromFailure(
      message || 'Apply failed',
      undefined,
      recipe
    );
    host.publishRuntimeDiagnostics?.(diagnostics);
    host.postMessage({
      type: EXTENSION_TO_WEBVIEW.error,
      message: message || 'Apply failed',
      diagnostics,
    });
  } finally {
    host.state.setSessionPhase(RUNNER_SESSION_PHASE.idle);
    host.postState();
  }
}

async function handleOpenDiffByPath(
  host: RecipeRunnerHandlerHost,
  filePath: string
): Promise<void> {
  const file = host.state.lastFiles.find((item) => item.path === filePath);
  if (!file) {
    return;
  }
  const materialized = await ensureDiffMaterialized(host, file);
  await openDiff(host, materialized);
}

function isNewFilePreview(file: FilePreview): boolean {
  if (file.isNew) {
    return true;
  }
  if (file.kind === FILE_PREVIEW_KIND.create) {
    return !(file.original ?? '').trim();
  }
  return false;
}

async function openNewFilePreview(
  host: RecipeRunnerHandlerHost,
  file: FilePreview
): Promise<void> {
  const safe = file.path.replace(/[^a-zA-Z0-9]/g, '_');
  const contentUri = host.diffProvider.store(
    `${DIFF.modifiedPrefix}/${safe}`,
    file.modified ?? ''
  );
  const doc = await vscode.workspace.openTextDocument(contentUri);
  await vscode.window.showTextDocument(doc, { preview: true });
}

async function openDiff(
  host: RecipeRunnerHandlerHost,
  file: FilePreview
): Promise<void> {
  if (isNewFilePreview(file)) {
    await openNewFilePreview(host, file);
    return;
  }

  const safe = file.path.replace(/[^a-zA-Z0-9]/g, '_');
  const originalText = file.original ?? '';
  const modifiedText = file.modified ?? '';
  const title = `${file.path} (proposed)`;

  const originalUri = host.diffProvider.store(
    `${DIFF.originalPrefix}/${safe}`,
    originalText
  );
  const modifiedUri = host.diffProvider.store(
    `${DIFF.modifiedPrefix}/${safe}`,
    modifiedText
  );
  await vscode.commands.executeCommand(
    'vscode.diff',
    originalUri,
    modifiedUri,
    title
  );
}

async function ensureDiffMaterialized(
  host: RecipeRunnerHandlerHost,
  file: FilePreview
): Promise<FilePreview> {
  if (file.original !== undefined && file.modified !== undefined) {
    return file;
  }

  const recipe = host.state.currentRecipe;
  if (!recipe) {
    return file;
  }

  const response = await host.bridge.diff(
    recipe.id,
    host.state.lastArgs,
    file.path
  );
  if (!response.ok || !response.file) {
    host.postMessage({
      type: EXTENSION_TO_WEBVIEW.error,
      message: response.error ?? `Failed to open diff for ${file.path}`,
    });
    return file;
  }

  const index = host.state.lastFiles.findIndex((item) => item.path === file.path);
  if (index >= 0) {
    host.state.lastFiles[index] = response.file;
    return host.state.lastFiles[index];
  }
  return response.file;
}
