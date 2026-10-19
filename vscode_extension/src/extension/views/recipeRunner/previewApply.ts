import * as path from 'path';
import * as vscode from 'vscode';
import {
  EXTENSION_TO_WEBVIEW,
  RUNNER_SESSION_PHASE,
  type ArgValues,
  type SelectionPayload,
  type RecipeDiagnostic,
  type RecipeSchema,
} from '../../../shared';
import type { RecipeRunnerHandlerHost } from './types';
import { refreshOpenDiffContents } from './diffPreview';

export async function handleOpenRecipeFile(
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

export async function handlePickPath(
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

export async function handlePreview(
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
    await refreshOpenDiffContents(host);
    if (!isCurrent()) {
      return;
    }
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

export function runtimeDiagnosticsFromFailure(
  message: string,
  fromHost: readonly RecipeDiagnostic[] | undefined,
  recipe: RecipeSchema
): RecipeDiagnostic[] {
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

export async function handleApply(
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

