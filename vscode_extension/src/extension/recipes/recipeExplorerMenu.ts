import * as vscode from 'vscode';
import type { ExtensionConfig } from '../config/extensionConfig';
import type { HostBridge } from '../host/hostBridge';
import type { RecipeRepository } from '../recipes/recipeRepository';
import type { RecipeRunnerViewProvider } from '../views/recipeRunnerViewProvider';
import type { RecipeSchema } from '../../shared';
import {
  prefillArgsFromUriClick,
  resolveUriContext,
  type ExplorerResourceKind,
} from './recipeContext';
import { invokeRecipe } from './recipeInvoke';

export async function runRecipeFromExplorer(
  deps: {
    repository: RecipeRepository;
    bridge: HostBridge;
    config: ExtensionConfig;
    runner: RecipeRunnerViewProvider;
  },
  resource?: vscode.Uri | { resourceUri?: vscode.Uri }
): Promise<void> {
  const uri = resolveExplorerUri(resource);
  if (!uri) {
    vscode.window.showWarningMessage(
      'Codemod Recipe: select a file or folder in the Explorer.'
    );
    return;
  }

  const kind = await detectExplorerKind(uri);
  const uriContext = resolveUriContext(uri, deps.config.workspaceRoot, kind);

  try {
    await deps.bridge.ensureHost();
    if (deps.repository.getRecipes().length === 0) {
      await deps.repository.reload();
    }
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    vscode.window.showErrorMessage(`Codemod Recipe: ${message}`);
    return;
  }

  let recipeIds: string[];
  try {
    const filtered = await deps.bridge.filterExplorerRecipes(
      uriContext.path,
      kind
    );
    if (!filtered.ok) {
      vscode.window.showErrorMessage(
        `Codemod Recipe: ${filtered.error ?? 'failed to filter explorer recipes'}`
      );
      return;
    }
    recipeIds = filtered.recipeIds ?? [];
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    vscode.window.showErrorMessage(`Codemod Recipe: ${message}`);
    return;
  }

  const recipes = recipeIds
    .map((id) => deps.repository.findById(id))
    .filter((r): r is RecipeSchema => Boolean(r));

  if (recipes.length === 0) {
    vscode.window.showInformationMessage(
      'Codemod Recipe: no recipes match this Explorer selection (explorerMenu).'
    );
    return;
  }

  const picked = await vscode.window.showQuickPick(
    recipes.map((recipe) => ({
      label: recipe.name,
      description: recipe.id,
      detail: recipe.description || undefined,
      recipe,
    })),
    {
      placeHolder: `Run recipe on ${uriContext.path}`,
      matchOnDescription: true,
      matchOnDetail: true,
    }
  );
  if (!picked) {
    return;
  }

  const clickPrefill = prefillArgsFromUriClick(
    picked.recipe,
    kind,
    uriContext.path
  );

  await invokeRecipe(
    { repository: deps.repository, bridge: deps.bridge, config: deps.config, runner: deps.runner },
    {
      recipeId: picked.recipe.id,
      mode: 'auto',
      args: clickPrefill,
      contextValues: uriContext.values,
      filePath: kind === 'file' ? uriContext.path : undefined,
    }
  );
}

function resolveExplorerUri(
  resource?: vscode.Uri | { resourceUri?: vscode.Uri }
): vscode.Uri | undefined {
  if (!resource) {
    const active = vscode.window.activeTextEditor?.document.uri;
    return active;
  }
  if (resource instanceof vscode.Uri) {
    return resource;
  }
  if (resource.resourceUri instanceof vscode.Uri) {
    return resource.resourceUri;
  }
  return undefined;
}

async function detectExplorerKind(
  uri: vscode.Uri
): Promise<ExplorerResourceKind> {
  try {
    const stat = await vscode.workspace.fs.stat(uri);
    if (stat.type & vscode.FileType.Directory) {
      return 'folder';
    }
  } catch {
    // Fall through — treat as file if stat fails.
  }
  return 'file';
}
