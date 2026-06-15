import * as vscode from 'vscode';
import { COMMANDS, VIEWS } from './constants';
import { ExtensionConfig } from './config/extensionConfig';
import { DiffContentProvider } from './diff/diffContentProvider';
import { DartBridge } from './host/dartBridge';
import { HostDiscovery } from './host/hostDiscovery';
import { prefillArgs, resolveEditorContext } from './recipes/recipeContext';
import { RecipeRepository } from './recipes/recipeRepository';
import type { RecipeSchema } from '../shared';
import { RecipeRunnerViewProvider } from './views/recipeRunnerViewProvider';

export function activate(context: vscode.ExtensionContext): void {
  const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  if (!workspaceRoot) {
    return;
  }

  const config = new ExtensionConfig();
  const hostDiscovery = new HostDiscovery(workspaceRoot, config);
  const bridge = new DartBridge(workspaceRoot, config, hostDiscovery);
  const diffProvider = new DiffContentProvider();
  const repository = new RecipeRepository(bridge);
  const runner = new RecipeRunnerViewProvider(
    bridge,
    config,
    diffProvider,
    workspaceRoot,
    context.extensionUri
  );

  const refreshAndSync = async (
    showError = false,
    options?: { restartHost?: boolean }
  ): Promise<void> => {
    const restartHost = options?.restartHost !== false;
    runner.setRecipesRefreshing(true);
    try {
      if (restartHost) {
        bridge.dispose();
        await bridge.ensureHost();
      }
      await repository.refresh();
      await runner.refreshRecipes(
        repository.getRecipes(),
        repository.getLastError()
      );
      const error = repository.getLastError();
      if (showError && error) {
        vscode.window.showWarningMessage(`Codemod Recipe: ${error}`);
      }
    } finally {
      runner.setRecipesRefreshing(false);
    }
  };

  const bootstrap = async (showError = false): Promise<void> => {
    runner.setBootstrap({ inFlight: true, phase: 'startingHost' });
    try {
      await bridge.ensureHost();
      runner.setBootstrap({ inFlight: true, phase: 'loadingRecipes' });
      await refreshAndSync(showError, { restartHost: false });
      runner.setBootstrap({ inFlight: false, phase: 'ready' });
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      runner.setBootstrap({ inFlight: false, phase: 'error', error: message });
    }
  };

  context.subscriptions.push(
    { dispose: () => bridge.dispose() },
    vscode.workspace.registerTextDocumentContentProvider(
      DiffContentProvider.scheme,
      diffProvider
    ),
    vscode.window.registerWebviewViewProvider(
      VIEWS.runner,
      runner,
      { webviewOptions: { retainContextWhenHidden: true } }
    ),
    vscode.commands.registerCommand(COMMANDS.refresh, () => refreshAndSync(true)),
    vscode.commands.registerCommand(COMMANDS.bootstrap, () => bootstrap(true)),
    vscode.commands.registerCommand(
      COMMANDS.runRecipe,
      async (recipe?: RecipeSchema) => {
        if (!recipe) {
          const picked = await vscode.window.showQuickPick(
            repository.getRecipes().map((item) => ({
              label: item.name,
              description: item.description,
              recipe: item,
            })),
            { placeHolder: 'Select a codemod recipe' }
          );
          recipe = picked?.recipe;
        }
        if (recipe) {
          runner.run(recipe);
        }
      }
    ),
    vscode.commands.registerCommand(
      COMMANDS.runFromCursorContext,
      async () => {
        const recipes = repository.getRecipes();
        const editorContext = resolveEditorContext(workspaceRoot);
        const candidates = recipes
          .map((recipe) => ({
            recipe,
            args: prefillArgs(recipe, editorContext.values),
          }))
          .filter((candidate) => Object.keys(candidate.args).length > 0);

        if (candidates.length === 0) {
          vscode.window.showInformationMessage(
            'No recipes declare arguments that match the current editor context.'
          );
          return;
        }

        const picked = await vscode.window.showQuickPick(
          candidates.map((candidate) => ({
            label: candidate.recipe.name,
            description: candidate.recipe.description,
            detail: Object.entries(candidate.args)
              .map(([key, value]) => `${key}: ${value}`)
              .join(', '),
            candidate,
          })),
          { placeHolder: 'Run recipe using values from the current cursor context' }
        );

        if (picked) {
          runner.run(picked.candidate.recipe, picked.candidate.args);
        }
      }
    ),
    vscode.commands.registerCommand(
      COMMANDS.configureHost,
      async () => {
        const value = await vscode.window.showInputBox({
          prompt:
            'Path (relative to workspace) of the Dart host entry point registering recipes',
          placeHolder: 'tool/codemod_host.dart',
        });
        if (value !== undefined) {
          await config.updateHostEntrypoint(value);
          bridge.dispose();
          await bootstrap(true);
        }
      }
    )
  );

  void bootstrap();
}

export function deactivate(): void {
  // No-op: child processes are short-lived and exit on their own.
}
