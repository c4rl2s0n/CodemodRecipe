import * as path from 'path';
import * as vscode from 'vscode';
import {
  createExtensionLifecycle,
  formatHostError,
  pickInvokeMode,
  pickRecipeId,
} from './bootstrap';
import { COMMANDS, VIEWS } from './constants';
import { ExtensionConfig } from './config/extensionConfig';
import { DiffContentProvider } from './diff/diffContentProvider';
import { HostBridge } from './host/hostBridge';
import { RecipeDiagnostics } from './language/diagnostics';
import { registerRecipeLanguageSupport, type LanguageSession } from './language/session';
import { resolveEditorContext } from './recipes/recipeContext';
import { matchRecipesToEditorContext } from './recipes/recipeContextMatch';
import { registerContextMatchUpdates } from './recipes/contextMatchService';
import { createShortcutForRecipe } from './recipes/createShortcut';
import {
  invokeRecipe,
  invokeSlot,
  type InvokeArgs,
  type InvokeSlotArgs,
} from './recipes/recipeInvoke';
import {
  formatInvokeKeybindingJson,
  formatSlotKeybindingJson,
} from './recipes/recipeKeybindings';
import { openRecipeFromExplorer, runRecipeFromExplorer } from './recipes/recipeExplorerMenu';
import { RecipeRepository } from './recipes/recipeRepository';
import type { RecipeSchema } from '../shared';
import { RecipeRunnerViewProvider } from './views/recipeRunnerViewProvider';
import { QueryToolsController } from './queryTools/queryToolsController';
import { CodemodTestController } from './testing/codemodTestController';

export function activate(context: vscode.ExtensionContext): void {
  const config = new ExtensionConfig();
  const bridge = new HostBridge(config, context.extensionUri);
  const diffProvider = new DiffContentProvider();
  const repository = new RecipeRepository(bridge);
  const diagnostics = new RecipeDiagnostics();
  const runner = new RecipeRunnerViewProvider(
    bridge,
    config,
    diffProvider,
    context.extensionUri
  );
  runner.setRecipeRepository(repository);
  runner.setRuntimeDiagnosticsPublisher((runtimeDiags) => {
    const discovery = repository.getDiagnostics();
    diagnostics.publish([...discovery, ...runtimeDiags], config.workspaceRoot);
  });
  const queryTools = new QueryToolsController(bridge, config);
  queryTools.register(context);
  const testController = new CodemodTestController(bridge);

  let languageSession: LanguageSession | undefined;
  const recreateLanguageSession = (): void => {
    languageSession?.dispose();
    languageSession = registerRecipeLanguageSupport(
      context,
      repository,
      bridge,
      config
    );
  };
  recreateLanguageSession();
  context.subscriptions.push({
    dispose: () => {
      languageSession?.dispose();
      languageSession = undefined;
    },
  });

  const life = createExtensionLifecycle({
    config,
    bridge,
    repository,
    diagnostics,
    runner,
    testController,
    recreateLanguageSession,
  });

  context.subscriptions.push(
    { dispose: () => bridge.dispose() },
    { dispose: () => life.disposeCodemodWatcher() },
    { dispose: () => testController.dispose() },
    diagnostics.disposable,
    registerContextMatchUpdates({ repository, config, runner }),
    vscode.workspace.onDidSaveTextDocument((document) => {
      if (document.uri.scheme !== 'file') {
        return;
      }
      if (life.isUnderCodemodRoot(document.uri.fsPath)) {
        life.scheduleRecipeReload();
        return;
      }
      const root = config.workspaceRoot;
      if (!root || !document.uri.fsPath.startsWith(root)) {
        return;
      }
      if (repository.getComponents().length === 0 || !repository.getStack()) {
        return;
      }
      const relative = path.relative(root, document.uri.fsPath).replace(/\\/g, '/');
      if (relative.startsWith('..') || path.isAbsolute(relative)) {
        return;
      }
      void runner.incrementalComponentScan([relative]);
    }),
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (
        event.affectsConfiguration('codemodRecipe.codemodRoot') ||
        event.affectsConfiguration('codemodRecipe.workspaceRoot')
      ) {
        life.onWorkspaceConfigChanged();
      }
    }),
    vscode.workspace.registerTextDocumentContentProvider(
      DiffContentProvider.scheme,
      diffProvider
    ),
    vscode.window.registerWebviewViewProvider(VIEWS.runner, runner, {
      webviewOptions: { retainContextWhenHidden: true },
    }),
    vscode.commands.registerCommand(COMMANDS.refresh, () =>
      life.reloadRecipesFromHost(true)
    ),
    vscode.commands.registerCommand(COMMANDS.validateRecipes, async () => {
      try {
        await bridge.ensureHost();
        const result = await bridge.validateRecipes();
        await repository.reload();
        await life.syncRunnerFromRepository();
        const diags = result.diagnostics ?? [];
        const errors = diags.filter((d) => d.severity === 'error');
        const warnings = diags.filter((d) => d.severity === 'warning');
        if (result.ok) {
          const suffix =
            warnings.length > 0 ? ` (${warnings.length} warning(s))` : '';
          vscode.window.showInformationMessage(
            `Codemod Recipe: validation passed${suffix}`
          );
        } else {
          const detail = errors
            .slice(0, 3)
            .map((d) => d.message)
            .join('; ');
          vscode.window.showErrorMessage(
            `Codemod Recipe: ${errors.length} validation error(s) — ${detail}`
          );
        }
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        vscode.window.showErrorMessage(
          formatHostError(`Codemod Recipe validate failed: ${message}`)
        );
      }
    }),
    vscode.commands.registerCommand(COMMANDS.runTests, async () => {
      try {
        await bridge.ensureHost();
        await testController.runAll();
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        vscode.window.showErrorMessage(
          formatHostError(`Codemod Recipe run tests failed: ${message}`)
        );
      }
    }),
    vscode.commands.registerCommand(COMMANDS.bootstrap, () =>
      life.bootstrap(true)
    ),
    vscode.commands.registerCommand(COMMANDS.scaffoldProject, () =>
      life.scaffoldProject(false)
    ),
    vscode.commands.registerCommand(COMMANDS.updateProjectScaffold, () =>
      life.updateProjectScaffold()
    ),
    vscode.commands.registerCommand(
      COMMANDS.runRecipe,
      async (recipe?: RecipeSchema, initialArgs?: Record<string, string>) => {
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
          runner.run(recipe, initialArgs ?? {});
        }
      }
    ),
    vscode.commands.registerCommand(
      COMMANDS.invoke,
      async (args?: InvokeArgs) => {
        await invokeRecipe(
          { repository, bridge, config, runner },
          args ?? {}
        );
      }
    ),
    vscode.commands.registerCommand(
      COMMANDS.invokeSlot,
      async (args?: InvokeSlotArgs) => {
        await invokeSlot({ repository, bridge, config, runner }, args ?? {});
      }
    ),
    vscode.commands.registerCommand(
      COMMANDS.copyInvokeKeybinding,
      async (recipeId?: string) => {
        const id =
          typeof recipeId === 'string'
            ? recipeId
            : await pickRecipeId(repository);
        if (!id) {
          return;
        }
        const mode = await pickInvokeMode('auto');
        if (!mode) {
          return;
        }
        await vscode.env.clipboard.writeText(formatInvokeKeybindingJson(id, mode));
        vscode.window.showInformationMessage(
          'Codemod Recipe: invoke keybinding JSON copied to clipboard.'
        );
      }
    ),
    vscode.commands.registerCommand(
      COMMANDS.assignToSlot,
      async (recipeId?: string) => {
        const id =
          typeof recipeId === 'string'
            ? recipeId
            : await pickRecipeId(repository);
        if (!id) {
          return;
        }
        await createShortcutForRecipe({ config, repository }, id);
      }
    ),
    vscode.commands.registerCommand(
      COMMANDS.copySlotKeybinding,
      async (slot?: string) => {
        const slotId =
          typeof slot === 'string' && slot.trim()
            ? slot.trim()
            : await vscode.window.showInputBox({
                prompt: 'Slot id to copy keybinding for',
                placeHolder: '1',
              });
        if (!slotId?.trim()) {
          return;
        }
        const mode = await pickInvokeMode('auto');
        if (!mode) {
          return;
        }
        await vscode.env.clipboard.writeText(
          formatSlotKeybindingJson(slotId.trim(), mode)
        );
        vscode.window.showInformationMessage(
          'Codemod Recipe: slot keybinding JSON copied to clipboard.'
        );
      }
    ),
    vscode.commands.registerCommand(COMMANDS.runFromCursorContext, async () => {
      const recipes = repository.getRecipes();
      const editorContext = resolveEditorContext(config.workspaceRoot);
      const candidates = matchRecipesToEditorContext(
        recipes,
        editorContext.values
      );

      if (candidates.length === 0) {
        vscode.window.showInformationMessage(
          'No recipes declare arguments that match the current editor context.'
        );
        return;
      }

      const picked = await vscode.window.showQuickPick(
        candidates.map((candidate) => ({
          label: candidate.name,
          description: candidate.description,
          detail: Object.entries(candidate.args)
            .map(([key, value]) => `${key}: ${value}`)
            .join(', '),
          candidate,
        })),
        {
          placeHolder:
            'Run recipe using values from the current cursor context',
        }
      );

      if (picked) {
        await invokeRecipe(
          { repository, bridge, config, runner },
          {
            recipeId: picked.candidate.recipeId,
            mode: 'auto',
            args: picked.candidate.args,
          }
        );
      }
    }),
    vscode.commands.registerCommand(
      COMMANDS.runFromExplorer,
      async (resource?: vscode.Uri) => {
        await runRecipeFromExplorer(
          { repository, bridge, config, runner },
          resource
        );
      }
    ),
    vscode.commands.registerCommand(
      COMMANDS.openFromExplorer,
      async (resource?: vscode.Uri) => {
        await openRecipeFromExplorer(
          { repository, bridge, config, runner },
          resource
        );
      }
    ),
    vscode.commands.registerCommand(COMMANDS.configureCodemodRoot, async () => {
      const value = await vscode.window.showInputBox({
        prompt: 'Path (relative to workspace) of the codemod root directory',
        placeHolder: '.codemod',
      });
      if (value !== undefined) {
        await config.updateCodemodRoot(value);
        bridge.dispose();
        await life.bootstrap(true);
      }
    })
  );

  runner.setScaffoldHandler(() => life.scaffoldProject(false));
  void life.bootstrap();
}

export function deactivate(): void {
  // No-op: child processes are short-lived and exit on their own.
}
