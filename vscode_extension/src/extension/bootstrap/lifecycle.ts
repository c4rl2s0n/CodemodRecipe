import * as fs from 'fs';
import * as path from 'path';
import * as vscode from 'vscode';
import type { ExtensionConfig } from '../config/extensionConfig';
import type { HostBridge } from '../host/hostBridge';
import type { RecipeDiagnostics } from '../language/diagnostics';
import type { RecipeRepository } from '../recipes/recipeRepository';
import { refreshContextMatches } from '../recipes/contextMatchService';
import type { RecipeRunnerViewProvider } from '../views/recipeRunnerViewProvider';
import type { CodemodTestController } from '../testing/codemodTestController';
import {
  configHasAbsoluteWorkspaceRoot,
  formatHostError,
  isEmptyCodemodDir,
} from './workspaceSetup';

export type LifecycleDeps = {
  config: ExtensionConfig;
  bridge: HostBridge;
  repository: RecipeRepository;
  diagnostics: RecipeDiagnostics;
  runner: RecipeRunnerViewProvider;
  testController: CodemodTestController;
  recreateLanguageSession: () => void;
};

export type ExtensionLifecycle = {
  bootstrap: (showError?: boolean) => Promise<void>;
  reloadRecipesFromHost: (showError?: boolean) => Promise<void>;
  scaffoldProject: (force: boolean) => Promise<void>;
  updateProjectScaffold: () => Promise<void>;
  syncRunnerFromRepository: () => Promise<void>;
  disposeCodemodWatcher: () => void;
  isUnderCodemodRoot: (fsPath: string) => boolean;
  scheduleRecipeReload: () => void;
  onWorkspaceConfigChanged: () => void;
};

/** Owns host reload, scaffold offer, and .codemod file watching. */
export function createExtensionLifecycle(
  deps: LifecycleDeps
): ExtensionLifecycle {
  const {
    config,
    bridge,
    repository,
    diagnostics,
    runner,
    testController,
    recreateLanguageSession,
  } = deps;

  let recipeReloadTimer: NodeJS.Timeout | undefined;
  let recipeReloadGeneration = 0;
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

  const syncRunnerFromRepository = async (): Promise<void> => {
    await runner.refreshRecipes(
      repository.getRecipes(),
      repository.getLastError(),
      repository.getDiagnostics()
    );
    runner.setComponentCatalog(
      repository.getComponents(),
      repository.getStack()
    );
    diagnostics.publish(repository.getDiagnostics(), config.workspaceRoot);
    refreshContextMatches({ repository, config, runner });
    await testController.syncFromCatalog(
      repository.getTestSuites(),
      config.workspaceRoot
    );
    if (repository.getComponents().length > 0 && repository.getStack()) {
      await runner.scanComponents();
    } else {
      runner.clearComponentScan();
    }
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

  const scheduleRecipeReload = (): void => {
    if (recipeReloadTimer) {
      clearTimeout(recipeReloadTimer);
    }
    const generation = ++recipeReloadGeneration;
    recipeReloadTimer = setTimeout(() => {
      void (async () => {
        await reloadRecipesFromHost();
        if (generation !== recipeReloadGeneration) {
          return;
        }
      })();
    }, 300);
  };

  const createCodemodWatcher = (): void => {
    disposeCodemodWatcher();
    const pattern = '**/*.{luau,lua,template,scm}';
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
        force
          ? `Codemod Recipe: updated project scaffold (${written} written, ${skipped} skipped).`
          : `Codemod Recipe: scaffolded project (${written} written, ${skipped} skipped).`
      );
      createCodemodWatcher();
      await reloadRecipesFromHost(true);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      vscode.window.showErrorMessage(
        formatHostError(`Scaffold failed: ${message}`)
      );
    }
  };

  const updateProjectScaffold = async (): Promise<void> => {
    const choice = await vscode.window.showWarningMessage(
      'Update Project Scaffold will overwrite existing Codemod scaffold files (skills, rules, stubs including codemod-context.d.luau, and recommended .vscode settings). Continue?',
      { modal: true },
      'Update'
    );
    if (choice !== 'Update') {
      return;
    }
    await scaffoldProject(true);
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

  const onWorkspaceConfigChanged = (): void => {
    bridge.dispose();
    recreateLanguageSession();
    void bootstrap(true);
  };

  return {
    bootstrap,
    reloadRecipesFromHost,
    scaffoldProject,
    updateProjectScaffold,
    syncRunnerFromRepository,
    disposeCodemodWatcher,
    isUnderCodemodRoot,
    scheduleRecipeReload,
    onWorkspaceConfigChanged,
  };
}
