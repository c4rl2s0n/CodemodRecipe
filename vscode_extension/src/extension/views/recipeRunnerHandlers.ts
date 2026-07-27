import * as path from 'path';
import * as vscode from 'vscode';
import { COMMANDS, DIFF } from '../constants';
import type { DiffContentProvider } from '../diff/diffContentProvider';
import type { ExtensionConfig } from '../config/extensionConfig';
import type { HostBridge } from '../host/hostBridge';
import {
  EXTENSION_TO_WEBVIEW,
  WEBVIEW_TO_EXTENSION,
  assertNever,
  FILE_PREVIEW_KIND,
  type FilePreview,
  type SelectionPayload,
  type WebviewToExtensionMessage,
} from '../../shared';
import type { RecipeRunnerState } from './recipeRunnerState';

export interface RecipeRunnerHandlerHost {
  readonly workspaceRoot: string;
  readonly state: RecipeRunnerState;
  readonly bridge: HostBridge;
  readonly config: ExtensionConfig;
  readonly diffProvider: DiffContentProvider;
  get previewInFlight(): boolean;
  setPreviewInFlight(value: boolean): void;
  get scaffoldHandler(): (() => Promise<void>) | undefined;
  postState(): void;
  postMessage(message: unknown): void;
  run(recipe: import('../../shared').RecipeSchema): void;
  ensureRecipeDetails(
    recipe: import('../../shared').RecipeSchema
  ): Promise<import('../../shared').RecipeSchema>;
  argsKey(args: Record<string, string>): string;
}

export async function handleWebviewMessage(
  host: RecipeRunnerHandlerHost,
  message: WebviewToExtensionMessage
): Promise<void> {
  switch (message.type) {
    case WEBVIEW_TO_EXTENSION.ready:
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
      await handlePickPath(host, message.arg, false);
      break;
    case WEBVIEW_TO_EXTENSION.pickDirectory:
      await handlePickPath(host, message.arg, true);
      break;
    case WEBVIEW_TO_EXTENSION.preview:
      await handlePreview(host, message.args, message.requestId);
      break;
    case WEBVIEW_TO_EXTENSION.openDiff:
      await handleOpenDiffByPath(host, message.path, message.patchIndex);
      break;
    case WEBVIEW_TO_EXTENSION.apply:
      await handleApply(host, message.selection);
      break;
    default:
      assertNever(message);
  }
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
  directory: boolean
): Promise<void> {
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
  if (!picked?.[0]) return;

  const rel = path.relative(host.workspaceRoot, picked[0].fsPath);
  host.postMessage({
    type: EXTENSION_TO_WEBVIEW.filePicked,
    arg,
    value: rel.startsWith('..') ? picked[0].fsPath : rel,
  });
}

async function handlePreview(
  host: RecipeRunnerHandlerHost,
  args: Record<string, string>,
  requestId?: number
): Promise<void> {
  const recipe = host.state.currentRecipe;
  if (!recipe) return;
  if (host.previewInFlight) {
    return;
  }

  host.setPreviewInFlight(true);
  host.postMessage({
    type: EXTENSION_TO_WEBVIEW.previewState,
    inFlight: true,
    requestId,
  });
  host.state.lastArgs = args;
  const argsKey = host.argsKey(args);
  try {
    const response = await host.bridge.preview(
      recipe.id,
      args,
      host.config.previewSnippetLines
    );
    if (!response.ok) {
      host.postMessage({
        type: EXTENSION_TO_WEBVIEW.error,
        message: response.error ?? 'Preview failed',
        requestId,
      });
      return;
    }

    host.state.lastFiles = response.files ?? [];
    host.state.lastPreviewToken = response.previewToken;
    host.postMessage({
      type: EXTENSION_TO_WEBVIEW.previewResult,
      files: host.state.lastFiles,
      requestId,
      argsKey,
    });
  } finally {
    host.setPreviewInFlight(false);
    host.postMessage({
      type: EXTENSION_TO_WEBVIEW.previewState,
      inFlight: false,
      requestId,
    });
  }
}

async function handleApply(
  host: RecipeRunnerHandlerHost,
  selection: SelectionPayload
): Promise<void> {
  const recipe = host.state.currentRecipe;
  if (!recipe) return;

  const previewToken = host.state.lastPreviewToken;
  if (!previewToken) {
    host.postMessage({
      type: EXTENSION_TO_WEBVIEW.error,
      message: 'Preview is out of date. Re-run preview before applying.',
    });
    return;
  }

  const response = await host.bridge.apply(
    recipe.id,
    host.state.lastArgs,
    selection,
    previewToken
  );
  if (!response.ok) {
    host.postMessage({
      type: EXTENSION_TO_WEBVIEW.error,
      message: response.error ?? 'Apply failed',
    });
    return;
  }

  const count = response.applied?.length ?? 0;
  vscode.window.showInformationMessage(`Applied ${recipe.name} to ${count} file(s).`);
  host.postMessage({
    type: EXTENSION_TO_WEBVIEW.applyResult,
    applied: response.applied ?? [],
  });
}

async function handleOpenDiffByPath(
  host: RecipeRunnerHandlerHost,
  filePath: string,
  patchIndex: number
): Promise<void> {
  const file = host.state.lastFiles.find((item) => item.path === filePath);
  if (!file) {
    return;
  }
  const materialized = await ensureDiffMaterialized(host, file);
  await openDiff(host, materialized, patchIndex);
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
  file: FilePreview,
  patchIndex: number
): Promise<void> {
  if (isNewFilePreview(file)) {
    await openNewFilePreview(host, file);
    return;
  }

  const safe = file.path.replace(/[^a-zA-Z0-9]/g, '_');
  let originalText = file.original ?? '';
  let modifiedText = file.modified ?? '';
  let title = file.path;

  if (patchIndex >= 0) {
    const patch = file.patches[patchIndex];
    if (patch) {
      originalText = originalText.slice(
        patch.offset,
        patch.offset + patch.length
      );
      modifiedText =
        patch.replacement ?? patch.replacementPreview ?? '';
      title = `${file.path} (change ${patchIndex + 1})`;
    }
  } else {
    title = `${file.path} (proposed)`;
  }

  const originalUri = host.diffProvider.store(
    `${DIFF.originalPrefix}/${safe}/${patchIndex}`,
    originalText
  );
  const modifiedUri = host.diffProvider.store(
    `${DIFF.modifiedPrefix}/${safe}/${patchIndex}`,
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
