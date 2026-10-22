import * as vscode from 'vscode';
import {
  diffModifiedKey,
  diffOriginalKey,
  diffSafePath,
} from '../../diff/diffContentProvider';
import {
  EXTENSION_TO_WEBVIEW,
  FILE_PREVIEW_KIND,
  type FilePreview,
} from '../../../shared';
import type { RecipeRunnerHandlerHost } from './types';

export async function handleOpenDiffByPath(
  host: RecipeRunnerHandlerHost,
  filePath: string
): Promise<void> {
  const file =
    host.state.lastFiles.find((item) => item.path === filePath) ??
    host.state.componentFiles.find((item) => item.path === filePath);
  if (!file) {
    return;
  }
  const materialized = await ensureDiffMaterialized(host, file);
  await openDiff(host, materialized);
}

export function isNewFilePreview(file: FilePreview): boolean {
  if (file.isNew) {
    return true;
  }
  if (file.kind === FILE_PREVIEW_KIND.create) {
    return !(file.original ?? '').trim();
  }
  return false;
}

export async function openNewFilePreview(
  host: RecipeRunnerHandlerHost,
  file: FilePreview
): Promise<void> {
  const contentUri = host.diffProvider.store(
    diffModifiedKey(file.path),
    file.modified ?? ''
  );
  const doc = await vscode.workspace.openTextDocument(contentUri);
  await vscode.window.showTextDocument(doc, { preview: true });
}

export async function openDiff(
  host: RecipeRunnerHandlerHost,
  file: FilePreview
): Promise<void> {
  if (isNewFilePreview(file)) {
    await openNewFilePreview(host, file);
    return;
  }

  const originalText = file.original ?? '';
  const modifiedText = file.modified ?? '';
  const title = `${file.path} (proposed)`;

  const originalUri = host.diffProvider.store(
    diffOriginalKey(file.path),
    originalText
  );
  const modifiedUri = host.diffProvider.store(
    diffModifiedKey(file.path),
    modifiedText
  );
  await vscode.commands.executeCommand(
    'vscode.diff',
    originalUri,
    modifiedUri,
    title
  );
}

/** Re-store content for diffs already opened so vscode.diff / preview docs live-update. */
export async function refreshOpenDiffContents(
  host: RecipeRunnerHandlerHost
): Promise<void> {
  const presentSafe = new Set<string>();

  for (const file of host.state.lastFiles) {
    const originalKey = diffOriginalKey(file.path);
    const modifiedKey = diffModifiedKey(file.path);
    presentSafe.add(diffSafePath(file.path));

    const opened =
      host.diffProvider.has(originalKey) || host.diffProvider.has(modifiedKey);
    if (!opened) {
      continue;
    }

    const materialized = await ensureDiffMaterialized(host, file);
    if (isNewFilePreview(materialized)) {
      host.diffProvider.store(modifiedKey, materialized.modified ?? '');
      if (host.diffProvider.has(originalKey)) {
        host.diffProvider.store(originalKey, '');
      }
    } else {
      host.diffProvider.store(originalKey, materialized.original ?? '');
      host.diffProvider.store(modifiedKey, materialized.modified ?? '');
    }
  }

  for (const key of host.diffProvider.keys()) {
    const match = key.match(/^\/(?:original|modified)\/(.+)$/);
    if (!match) {
      continue;
    }
    if (!presentSafe.has(match[1])) {
      host.diffProvider.store(key, '');
    }
  }
}

export async function ensureDiffMaterialized(
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
