import * as path from 'path';
import * as vscode from 'vscode';
import { COMMANDS, DIFF, EXTENSION, VIEWS } from '../constants';
import { DiffContentProvider } from '../diff/diffContentProvider';
import { ExtensionConfig } from '../config/extensionConfig';
import { DartBridge } from '../host/dartBridge';
import {
  EXTENSION_TO_WEBVIEW,
  WEBVIEW_TO_EXTENSION,
  isWebviewToExtensionMessage,
  type FilePreview,
  type RecipeDiagnostic,
  type RecipeSchema,
  type SelectionPayload,
} from '../../shared';
import { renderRecipeViewHtml } from '../webview/recipeViewHtml';
import { RecipeRunnerState } from './recipeRunnerState';

export class RecipeRunnerViewProvider implements vscode.WebviewViewProvider {
  private view: vscode.WebviewView | undefined;
  private renderPending = false;
  private webviewHtmlLoaded = false;
  private previewInFlight = false;
  private readonly state = new RecipeRunnerState();

  constructor(
    private readonly bridge: DartBridge,
    private readonly config: ExtensionConfig,
    private readonly diffProvider: DiffContentProvider,
    private readonly workspaceRoot: string,
    private readonly extensionUri: vscode.Uri
  ) { }

  resolveWebviewView(webviewView: vscode.WebviewView): void {
    this.view = webviewView;
    webviewView.webview.options = {
      enableScripts: true,
      localResourceRoots: [vscode.Uri.joinPath(this.extensionUri, 'media')],
    };
    const hadPendingRender = this.renderPending;
    this.ensureWebviewHtml();
    this.postState();
    if (hadPendingRender) {
      this.postState();
    }
    webviewView.webview.onDidReceiveMessage((message: unknown) => {
      void this.handleMessage(message);
    });
  }

  setRecipes(
    recipes: readonly RecipeSchema[],
    error?: string,
    diagnostics: readonly RecipeDiagnostic[] = []
  ): void {
    this.state.setRecipes(recipes, error, diagnostics);
    this.postState();
  }

  async refreshRecipes(
    recipes: readonly RecipeSchema[],
    error?: string,
    diagnostics: readonly RecipeDiagnostic[] = []
  ): Promise<void> {
    this.state.syncRecipesAfterRefresh(recipes, error, diagnostics);
    if (this.state.currentRecipe) {
      this.state.currentRecipe = await this.ensureRecipeDetails(this.state.currentRecipe);
    }
    this.diffProvider.clear();
    this.postState();
  }

  setRecipesRefreshing(inFlight: boolean): void {
    this.state.setRecipesRefreshing(inFlight);
    this.postState();
  }

  setBootstrap(state: {
    inFlight: boolean;
    phase: 'startingHost' | 'loadingRecipes' | 'ready' | 'error';
    error?: string;
  }): void {
    this.state.setBootstrap(state);
    this.postState();
  }

  run(recipe: RecipeSchema, initialArgs: Record<string, string> = {}): void {
    void this.runInternal(recipe, initialArgs);
  }

  private async handleMessage(message: unknown): Promise<void> {
    if (!isWebviewToExtensionMessage(message)) return;

    switch (message.type) {
      case WEBVIEW_TO_EXTENSION.ready:
        // The webview may miss early messages if we post state before its script is loaded.
        // Re-send the latest state once the webview confirms it is ready.
        this.postState();
        break;
      case WEBVIEW_TO_EXTENSION.bootstrapRetry:
        await vscode.commands.executeCommand(COMMANDS.bootstrap);
        break;
      case WEBVIEW_TO_EXTENSION.selectRecipe:
        await this.selectRecipe(message.id);
        break;
      case WEBVIEW_TO_EXTENSION.refreshRecipes:
        await vscode.commands.executeCommand(COMMANDS.refresh);
        break;
      case WEBVIEW_TO_EXTENSION.configureHost:
        await vscode.commands.executeCommand(COMMANDS.configureCodemodRoot);
        break;
      case WEBVIEW_TO_EXTENSION.pickFile:
        await this.pickPath(message.arg, false);
        break;
      case WEBVIEW_TO_EXTENSION.pickDirectory:
        await this.pickPath(message.arg, true);
        break;
      case WEBVIEW_TO_EXTENSION.preview:
        await this.preview(message.args, message.requestId);
        break;
      case WEBVIEW_TO_EXTENSION.openDiff:
        await this.openDiffByPath(message.path);
        break;
      case WEBVIEW_TO_EXTENSION.apply:
        await this.apply(message.selection);
        break;
    }
  }

  private async selectRecipe(recipeId: string): Promise<void> {
    const recipe = this.state.recipes.find((item) => item.id === recipeId);
    if (recipe) {
      this.run(recipe);
    }
  }

  private async runInternal(
    recipe: RecipeSchema,
    initialArgs: Record<string, string>
  ): Promise<void> {
    const hydrated = await this.ensureRecipeDetails(recipe);
    this.state.selectRecipe(hydrated, initialArgs);
    await this.revealAndPostState();
  }

  private async ensureRecipeDetails(recipe: RecipeSchema): Promise<RecipeSchema> {
    if (recipe.templatesLoaded !== false) {
      return recipe;
    }
    try {
      return await this.bridge.describe(recipe.id);
    } catch {
      return recipe;
    }
  }

  private async pickPath(arg: string, directory: boolean): Promise<void> {
    const picked = await vscode.window.showOpenDialog({
      defaultUri: vscode.Uri.parse(this.workspaceRoot),
      canSelectMany: false,
      canSelectFiles: !directory,
      canSelectFolders: directory,
      filters: {"Dart": ["dart"]},
      openLabel: 'Select',
    });
    if (!picked?.[0]) return;

    const rel = path.relative(this.workspaceRoot, picked[0].fsPath);
    this.postMessage({
      type: EXTENSION_TO_WEBVIEW.filePicked,
      arg,
      value: rel.startsWith('..') ? picked[0].fsPath : rel,
    });
  }

  private async preview(
    args: Record<string, string>,
    requestId?: number
  ): Promise<void> {
    const recipe = this.state.currentRecipe;
    if (!recipe) return;
    if (this.previewInFlight) {
      return;
    }

    this.previewInFlight = true;
    this.postMessage({
      type: EXTENSION_TO_WEBVIEW.previewState,
      inFlight: true,
      requestId,
    });
    this.state.lastArgs = args;
    const argsKey = this.argsKey(args);
    try {
      const response = await this.bridge.preview(
        recipe.id,
        args,
        this.config.previewSnippetLines
      );
      if (!response.ok) {
        this.postMessage({
          type: EXTENSION_TO_WEBVIEW.error,
          message: response.error ?? 'Preview failed',
          requestId,
        });
        return;
      }

      this.state.lastFiles = response.files ?? [];
      this.postMessage({
        type: EXTENSION_TO_WEBVIEW.previewResult,
        files: this.state.lastFiles,
        requestId,
        argsKey,
      });
    } finally {
      this.previewInFlight = false;
      this.postMessage({
        type: EXTENSION_TO_WEBVIEW.previewState,
        inFlight: false,
        requestId,
      });
    }
  }

  private async apply(selection: SelectionPayload): Promise<void> {
    const recipe = this.state.currentRecipe;
    if (!recipe) return;

    const response = await this.bridge.apply(
      recipe.id,
      this.state.lastArgs,
      selection
    );
    if (!response.ok) {
      this.postMessage({
        type: EXTENSION_TO_WEBVIEW.error,
        message: response.error ?? 'Apply failed',
      });
      return;
    }

    const count = response.applied?.length ?? 0;
    vscode.window.showInformationMessage(`Applied ${recipe.name} to ${count} file(s).`);
    this.postMessage({
      type: EXTENSION_TO_WEBVIEW.applyResult,
      applied: response.applied ?? [],
    });
  }

  private async openDiffByPath(filePath: string): Promise<void> {
    const file = this.state.lastFiles.find((item) => item.path === filePath);
    if (!file) {
      return;
    }
    const materialized = await this.ensureDiffMaterialized(file);
    await this.openDiff(materialized);
  }

  private async openDiff(file: FilePreview): Promise<void> {
    const safe = file.path.replace(/[^a-zA-Z0-9]/g, '_');
    const originalUri = this.diffProvider.store(
      `${DIFF.originalPrefix}/${safe}`,
      file.original ?? ''
    );
    const modifiedUri = this.diffProvider.store(
      `${DIFF.modifiedPrefix}/${safe}`,
      file.modified ?? ''
    );
    await vscode.commands.executeCommand(
      'vscode.diff',
      originalUri,
      modifiedUri,
      file.isNew ? `${file.path} (new)` : `${file.path} (proposed)`
    );
  }

  private async ensureDiffMaterialized(file: FilePreview): Promise<FilePreview> {
    if (file.original !== undefined && file.modified !== undefined) {
      return file;
    }

    const recipe = this.state.currentRecipe;
    if (!recipe) {
      return file;
    }

    const response = await this.bridge.diff(
      recipe.id,
      this.state.lastArgs,
      file.path
    );
    if (!response.ok || !response.file) {
      this.postMessage({
        type: EXTENSION_TO_WEBVIEW.error,
        message: response.error ?? `Failed to open diff for ${file.path}`,
      });
      return file;
    }

    const index = this.state.lastFiles.findIndex((item) => item.path === file.path);
    if (index >= 0) {
      this.state.lastFiles[index] = response.file;
      return this.state.lastFiles[index];
    }
    return response.file;
  }

  private async revealAndPostState(): Promise<void> {
    await vscode.commands.executeCommand(EXTENSION.activityViewId);
    try {
      await vscode.commands.executeCommand(`${VIEWS.runner}.focus`);
    } catch {
      // Some VS Code versions do not expose generated focus commands reliably.
    }
    this.ensureWebviewHtml();
    this.postState();
  }

  private ensureWebviewHtml(): void {
    if (!this.view || this.webviewHtmlLoaded) {
      return;
    }
    this.webviewHtmlLoaded = true;
    this.renderPending = false;
    this.view.webview.html = renderRecipeViewHtml(
      this.view.webview,
      this.extensionUri,
      { autoPreviewDebounceMs: this.config.autoPreviewDebounceMs }
    );
  }

  private postState(): void {
    if (!this.view) {
      this.renderPending = true;
      return;
    }
    this.renderPending = false;
    this.ensureWebviewHtml();
    this.postMessage({
      type: EXTENSION_TO_WEBVIEW.state,
      state: {
        ...this.state.toWebviewState(),
        autoPreviewDebounceMs: this.config.autoPreviewDebounceMs,
      },
    });
  }

  private postMessage(message: unknown): void {
    void this.view?.webview.postMessage(message);
  }

  private argsKey(args: Record<string, string>): string {
    const keys = Object.keys(args).sort();
    const ordered: Record<string, string> = {};
    for (const key of keys) {
      ordered[key] = args[key];
    }
    return JSON.stringify(ordered);
  }
}
