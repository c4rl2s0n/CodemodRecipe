import * as vscode from 'vscode';
import type { ExtensionConfig } from '../config/extensionConfig';
import type { HostBridge } from '../host/hostBridge';
import type { RecipeRepository } from '../recipes/recipeRepository';
import type { RecipeRunnerViewProvider } from '../views/recipeRunnerViewProvider';
import type { ExplorerRecipeMatch, RecipeSchema } from '../../shared';
import {
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

  let matches: ExplorerRecipeMatch[];
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
    matches = filtered.matches ?? [];
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    vscode.window.showErrorMessage(`Codemod Recipe: ${message}`);
    return;
  }

  const items = matches
    .map((match) => {
      const recipe = deps.repository.findById(match.recipeId);
      if (!recipe) {
        return undefined;
      }
      return { recipe, match };
    })
    .filter((item): item is { recipe: RecipeSchema; match: ExplorerRecipeMatch } =>
      Boolean(item)
    );

  if (items.length === 0) {
    vscode.window.showInformationMessage(
      'Codemod Recipe: no recipes match this Explorer selection (explorerMenu).'
    );
    return;
  }

  const picked = await vscode.window.showQuickPick(
    items.map(({ recipe, match }) => {
      const argDetail = Object.entries(match.args)
        .map(([key, value]) => `${key}: ${value}`)
        .join(', ');
      return {
        label: recipe.name,
        description: recipe.id,
        detail: argDetail || recipe.description || undefined,
        recipe,
        match,
      };
    }),
    {
      placeHolder: `Run recipe on ${uriContext.path}`,
      matchOnDescription: true,
      matchOnDetail: true,
    }
  );
  if (!picked) {
    return;
  }

  await invokeRecipe(
    {
      repository: deps.repository,
      bridge: deps.bridge,
      config: deps.config,
      runner: deps.runner,
    },
    {
      recipeId: picked.recipe.id,
      mode: 'auto',
      args: picked.match.args,
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
