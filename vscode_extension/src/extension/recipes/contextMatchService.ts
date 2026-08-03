import * as vscode from 'vscode';
import type { ExtensionConfig } from '../config/extensionConfig';
import type { RecipeRepository } from '../recipes/recipeRepository';
import type { RecipeRunnerViewProvider } from '../views/recipeRunnerViewProvider';
import { resolveEditorContext } from './recipeContext';
import { matchRecipesToEditorContext } from './recipeContextMatch';
import { slotsByRecipeId } from './recipeSlots';

const DEBOUNCE_MS = 200;

export function registerContextMatchUpdates(deps: {
  repository: RecipeRepository;
  config: ExtensionConfig;
  runner: RecipeRunnerViewProvider;
}): vscode.Disposable {
  let timer: ReturnType<typeof setTimeout> | undefined;
  let disposed = false;

  const schedule = () => {
    if (disposed) {
      return;
    }
    if (timer) {
      clearTimeout(timer);
    }
    timer = setTimeout(() => {
      timer = undefined;
      refresh();
    }, DEBOUNCE_MS);
  };

  const refresh = () => {
    if (disposed) {
      return;
    }
    const editorContext = resolveEditorContext(deps.config.workspaceRoot);
    const matches = matchRecipesToEditorContext(
      deps.repository.getRecipes(),
      editorContext.values
    );
    const slots = slotsByRecipeId(deps.config.structuredSlots);
    deps.runner.setContextMatching(matches, slots);
  };

  const subs = [
    vscode.window.onDidChangeActiveTextEditor(() => schedule()),
    vscode.window.onDidChangeTextEditorSelection(() => schedule()),
    vscode.workspace.onDidChangeConfiguration((e) => {
      if (e.affectsConfiguration('codemodRecipe.slots')) {
        schedule();
      }
    }),
  ];

  // Initial push once recipes may already be loaded.
  schedule();

  return new vscode.Disposable(() => {
    disposed = true;
    if (timer) {
      clearTimeout(timer);
    }
    for (const sub of subs) {
      sub.dispose();
    }
  });
}

/** Call after recipe catalog refresh so Context list stays current. */
export function refreshContextMatches(deps: {
  repository: RecipeRepository;
  config: ExtensionConfig;
  runner: RecipeRunnerViewProvider;
}): void {
  const editorContext = resolveEditorContext(deps.config.workspaceRoot);
  const matches = matchRecipesToEditorContext(
    deps.repository.getRecipes(),
    editorContext.values
  );
  const slots = slotsByRecipeId(deps.config.structuredSlots);
  deps.runner.setContextMatching(matches, slots);
}
