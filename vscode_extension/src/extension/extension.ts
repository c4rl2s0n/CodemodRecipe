import * as fs from 'fs';
import * as path from 'path';
import * as vscode from 'vscode';
import { COMMANDS, VIEWS } from './constants';
import { ExtensionConfig } from './config/extensionConfig';
import { DiffContentProvider } from './diff/diffContentProvider';
import { HostBridge } from './host/hostBridge';
import { RecipeDiagnostics } from './language/diagnostics';
import { registerRecipeLanguageSupport } from './language/recipeLanguage';
import { prefillArgs, resolveEditorContext } from './recipes/recipeContext';
import {
  formatInvokeKeybindingJson,
  formatSlotKeybindingJson,
  invokeRecipe,
  invokeSlot,
  type InvokeArgs,
  type InvokeMode,
  type InvokeSlotArgs,
} from './recipes/recipeInvoke';
import { openRecipeFromExplorer, runRecipeFromExplorer } from './recipes/recipeExplorerMenu';
import { RecipeRepository } from './recipes/recipeRepository';
import type { RecipeSchema } from '../shared';
import { RecipeRunnerViewProvider } from './views/recipeRunnerViewProvider';

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

  let recipeReloadTimer: NodeJS.Timeout | undefined;
  let codemodWatcher: vscode.FileSystemWatcher | undefined;
  let scaffoldOfferShown = false;

  const hasUsableWorkspaceRoot = (): boolean => {
    const root = config.workspaceRoot;
    if (!root || root === '.') {
      return false;
    }
    return Boolean(
      vscode.workspace.workspaceFolders?.length || configHasAbsoluteWorkspaceRoot()
    );
  };

  registerYamlSchemas(context);
  if (hasUsableWorkspaceRoot()) {
    registerRecipeLanguageSupport(context, repository, bridge, config);
  }

  const syncRunnerFromRepository = async (): Promise<void> => {
    await runner.refreshRecipes(
      repository.getRecipes(),
      repository.getLastError(),
      repository.getDiagnostics()
    );
    diagnostics.publish(repository.getDiagnostics(), config.workspaceRoot);
  };

  const maybeOfferScaffold = async (): Promise<void> => {
    if (scaffoldOfferShown) {
      return;
    }
    const root = config.workspaceRoot;
    const codemodDir = path.join(root, config.codemodRoot);
    const missingOrEmpty =
      !fs.existsSync(codemodDir) || isEmptyCodemodDir(codemodDir);
    if (!missingOrEmpty) {
      return;
    }
    scaffoldOfferShown = true;
    const choice = await vscode.window.showInformationMessage(
      `Codemod Recipe: no recipes found under ${config.codemodRoot}. Scaffold .codemod now?`,
      'Scaffold .codemod',
      'Not now'
    );
    if (choice === 'Scaffold .codemod') {
      await scaffoldProject(false);
    }
  };

  const reloadRecipesFromHost = async (showError = false): Promise<void> => {
    runner.setRecipesRefreshing(true);
    try {
      await bridge.ensureHost();
      try {
        await repository.reload();
      } catch {
        bridge.dispose();
        await bridge.ensureHost();
        await repository.refresh();
      }
      await syncRunnerFromRepository();
      if (showError && repository.getLastError()) {
        vscode.window.showWarningMessage(
          formatHostError(repository.getLastError()!)
        );
      }
      if (
        !repository.getLastError() &&
        repository.getRecipes().length === 0 &&
        repository.getDiagnostics().length === 0
      ) {
        await maybeOfferScaffold();
      }
    } finally {
      runner.setRecipesRefreshing(false);
    }
  };

  const restartHostAndRefresh = async (showError = false): Promise<void> => {
    runner.setRecipesRefreshing(true);
    try {
      bridge.dispose();
      await bridge.ensureHost();
      await repository.refresh();
      await syncRunnerFromRepository();
      if (showError && repository.getLastError()) {
        vscode.window.showWarningMessage(
          formatHostError(repository.getLastError()!)
        );
      }
    } finally {
      runner.setRecipesRefreshing(false);
    }
  };

  const scaffoldProject = async (force: boolean): Promise<void> => {
    try {
      await bridge.ensureHost();
      const result = await bridge.bootstrapProject(force);
      if (!result.ok) {
        vscode.window.showErrorMessage(
          `Codemod Recipe scaffold failed: ${result.error ?? 'unknown error'}`
        );
        return;
      }
      const written = result.written?.length ?? 0;
      const skipped = result.skipped?.length ?? 0;
      vscode.window.showInformationMessage(
        `Codemod Recipe: scaffolded project (${written} written, ${skipped} skipped).`
      );
      // Scaffold may have created the codemod root; recreate the watcher so
      // subsequent create/change events are observed.
      createCodemodWatcher();
      await reloadRecipesFromHost(true);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      vscode.window.showErrorMessage(
        formatHostError(`Scaffold failed: ${message}`)
      );
    }
  };

  const scheduleRecipeReload = (): void => {
    if (recipeReloadTimer) {
      clearTimeout(recipeReloadTimer);
    }
    recipeReloadTimer = setTimeout(() => {
      void reloadRecipesFromHost();
    }, 300);
  };

  const disposeCodemodWatcher = (): void => {
    codemodWatcher?.dispose();
    codemodWatcher = undefined;
  };

  const isUnderCodemodRoot = (fsPath: string): boolean => {
    const root = path.resolve(config.workspaceRoot, config.codemodRoot);
    const resolved = path.resolve(fsPath);
    return resolved === root || resolved.startsWith(root + path.sep);
  };

  const createCodemodWatcher = (): void => {
    disposeCodemodWatcher();
    const pattern = '**/*.{yaml,yml,template,scm}';
    const folder = vscode.workspace.workspaceFolders?.[0];
    const relativePattern =
      folder && !path.isAbsolute(config.codemodRoot)
        ? new vscode.RelativePattern(
            folder,
            path.posix.join(config.codemodRoot.replace(/\\/g, '/'), pattern)
          )
        : new vscode.RelativePattern(
            path.join(config.workspaceRoot, config.codemodRoot),
            pattern
          );

    codemodWatcher = vscode.workspace.createFileSystemWatcher(relativePattern);
    codemodWatcher.onDidChange(scheduleRecipeReload);
    codemodWatcher.onDidCreate(scheduleRecipeReload);
    codemodWatcher.onDidDelete(scheduleRecipeReload);
  };

  const bootstrap = async (showError = false): Promise<void> => {
    if (!hasUsableWorkspaceRoot()) {
      runner.setBootstrap({
        inFlight: false,
        phase: 'error',
        error:
          'Open a workspace folder (or set codemodRecipe.workspaceRoot to an absolute path), then click Retry.',
      });
      return;
    }

    runner.setBootstrap({ inFlight: true, phase: 'startingHost' });
    try {
      createCodemodWatcher();
      runner.setBootstrap({ inFlight: true, phase: 'loadingRecipes' });
      await restartHostAndRefresh(showError);
      runner.setBootstrap({ inFlight: false, phase: 'ready' });
      if (
        !repository.getLastError() &&
        repository.getRecipes().length === 0
      ) {
        await maybeOfferScaffold();
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      runner.setBootstrap({
        inFlight: false,
        phase: 'error',
        error: formatHostError(message),
      });
    }
  };

  context.subscriptions.push(
    { dispose: () => bridge.dispose() },
    { dispose: () => disposeCodemodWatcher() },
    diagnostics.disposable,
    vscode.workspace.onDidSaveTextDocument((document) => {
      if (document.uri.scheme !== 'file') {
        return;
      }
      if (isUnderCodemodRoot(document.uri.fsPath)) {
        scheduleRecipeReload();
      }
    }),
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (
        event.affectsConfiguration('codemodRecipe.codemodRoot') ||
        event.affectsConfiguration('codemodRecipe.workspaceRoot')
      ) {
        bridge.dispose();
        void bootstrap(true);
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
      reloadRecipesFromHost(true)
    ),
    vscode.commands.registerCommand(COMMANDS.validateRecipes, async () => {
      try {
        await bridge.ensureHost();
        const result = await bridge.validateRecipes();
        await repository.reload();
        await syncRunnerFromRepository();
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
    vscode.commands.registerCommand(COMMANDS.bootstrap, () => bootstrap(true)),
    vscode.commands.registerCommand(COMMANDS.scaffoldProject, () =>
      scaffoldProject(false)
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
        const slot = await vscode.window.showInputBox({
          prompt: 'Slot id (any character key, e.g. 1, b, c)',
          placeHolder: 'b',
          validateInput: (value) =>
            value.trim() ? undefined : 'Slot id is required',
        });
        if (!slot?.trim()) {
          return;
        }
        const normalized = slot.trim();
        await config.updateSlot(normalized, id);
        const copy = await vscode.window.showInformationMessage(
          `Codemod Recipe: assigned slot \`${normalized}\` → ${id}`,
          'Copy run keybinding',
          'Copy open keybinding'
        );
        if (copy === 'Copy run keybinding') {
          await vscode.env.clipboard.writeText(
            formatSlotKeybindingJson(normalized, 'auto')
          );
        } else if (copy === 'Copy open keybinding') {
          await vscode.env.clipboard.writeText(
            formatSlotKeybindingJson(normalized, 'open')
          );
        }
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
        {
          placeHolder:
            'Run recipe using values from the current cursor context',
        }
      );

      if (picked) {
        await invokeRecipe(
          { repository, bridge, config, runner },
          {
            recipeId: picked.candidate.recipe.id,
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
        await bootstrap(true);
      }
    })
  );

  // Allow webview scaffold messages to reach the same handler.
  runner.setScaffoldHandler(() => scaffoldProject(false));

  // Keep loading overlay until bootstrap finishes (state starts inFlight=true).
  void bootstrap();
}

export function deactivate(): void {
  // No-op: child processes are short-lived and exit on their own.
}

function configHasAbsoluteWorkspaceRoot(): boolean {
  const configured =
    vscode.workspace
      .getConfiguration('codemodRecipe')
      .get<string>('workspaceRoot') || '';
  return Boolean(configured && path.isAbsolute(configured));
}

function isEmptyCodemodDir(dir: string): boolean {
  try {
    const entries = fs.readdirSync(dir);
    if (entries.length === 0) {
      return true;
    }
    // Treat a tree with only empty recipe/map dirs as empty.
    const hasYaml = walkHasYaml(dir);
    return !hasYaml;
  } catch {
    return true;
  }
}

function walkHasYaml(dir: string, depth = 0): boolean {
  if (depth > 6) {
    return false;
  }
  let entries: string[];
  try {
    entries = fs.readdirSync(dir);
  } catch {
    return false;
  }
  for (const entry of entries) {
    const full = path.join(dir, entry);
    let stat: fs.Stats;
    try {
      stat = fs.statSync(full);
    } catch {
      continue;
    }
    if (stat.isFile() && /\.(ya?ml|template)$/i.test(entry)) {
      return true;
    }
    if (stat.isDirectory() && walkHasYaml(full, depth + 1)) {
      return true;
    }
  }
  return false;
}

function formatHostError(message: string): string {
  const prefixed = message.startsWith('Codemod Recipe')
    ? message
    : `Codemod Recipe: ${message}`;
  if (/build\.sh|cargo build|codemod_host|Failed to spawn|failed to start|Persistent host/i.test(message)) {
    if (/build\.sh|cargo/i.test(message)) {
      return prefixed;
    }
    return `${prefixed}\n\nIf the host failed to start, build it via vscode_extension/build.sh or cargo build -p codemod_recipe_host --bin codemod_host.`;
  }
  return prefixed;
}

async function pickRecipeId(
  repository: RecipeRepository
): Promise<string | undefined> {
  const picked = await vscode.window.showQuickPick(
    repository.getRecipes().map((item) => ({
      label: item.id,
      description: item.name,
      detail: item.description,
    })),
    { placeHolder: 'Select a recipe' }
  );
  return picked?.label;
}

async function pickInvokeMode(
  defaultMode: InvokeMode
): Promise<InvokeMode | undefined> {
  const picked = await vscode.window.showQuickPick(
    [
      {
        label: 'auto',
        description: 'Execute when complete; otherwise open runner',
      },
      { label: 'run', description: 'Prefer execute; open runner if incomplete' },
      { label: 'open', description: 'Always open recipe runner prefilled' },
    ],
    {
      placeHolder: `Invoke mode (default: ${defaultMode})`,
    }
  );
  return (picked?.label as InvokeMode | undefined) ?? undefined;
}

function registerYamlSchemas(context: vscode.ExtensionContext): void {
  const recipeSchema = vscode.Uri.joinPath(
    context.extensionUri,
    'schemas',
    'recipe.schema.json'
  ).toString();
  const mapSchema = vscode.Uri.joinPath(
    context.extensionUri,
    'schemas',
    'map.schema.json'
  ).toString();
  const variablesSchema = vscode.Uri.joinPath(
    context.extensionUri,
    'schemas',
    'variables.schema.json'
  ).toString();

  try {
    const yamlConfig = vscode.workspace.getConfiguration('yaml');
    const current = yamlConfig.get<Record<string, string | string[]>>('schemas') ?? {};
    const next: Record<string, string | string[]> = { ...current };
    let changed = false;
    const ensure = (schemaUri: string, patterns: string[]) => {
      if (next[schemaUri] === undefined) {
        next[schemaUri] = patterns;
        changed = true;
      }
    };
    ensure(recipeSchema, ['.codemod/**/*.yaml', '.codemod/**/*.yml']);
    ensure(mapSchema, ['.codemod/maps/**/*.yaml', '.codemod/maps/**/*.yml']);
    ensure(variablesSchema, [
      '.codemod/variables/**/*.yaml',
      '.codemod/variables/**/*.yml',
    ]);
    if (changed) {
      void yamlConfig.update(
        'schemas',
        next,
        vscode.ConfigurationTarget.Workspace
      );
    }
  } catch {
    // Red Hat YAML extension may be absent; jsonValidation still contributes.
  }
}
