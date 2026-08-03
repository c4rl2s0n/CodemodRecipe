import * as vscode from 'vscode';
import { EXTENSION, VIEWS } from '../constants';
import { DiffContentProvider } from '../diff/diffContentProvider';
import { ExtensionConfig } from '../config/extensionConfig';
import { HostBridge } from '../host/hostBridge';
import {
  EXTENSION_TO_WEBVIEW,
  isWebviewToExtensionMessage,
  type RecipeDiagnostic,
  type RecipeSchema,
} from '../../shared';
import { renderRecipeViewHtml } from '../webview/recipeViewHtml';
import { RecipeRunnerState } from './recipeRunnerState';
import {
  handleWebviewMessage,
  type RecipeRunnerHandlerHost,
} from './recipeRunnerHandlers';
import type { RecipeRepository } from '../recipes/recipeRepository';

export class RecipeRunnerViewProvider
  implements vscode.WebviewViewProvider, RecipeRunnerHandlerHost
{
  private view: vscode.WebviewView | undefined;
  private renderPending = false;
  private webviewHtmlLoaded = false;
  private _previewInFlight = false;
  readonly state = new RecipeRunnerState();
  private _scaffoldHandler: (() => Promise<void>) | undefined;
  private _recipeRepository: RecipeRepository | undefined;

  constructor(
    readonly bridge: HostBridge,
    readonly config: ExtensionConfig,
    readonly diffProvider: DiffContentProvider,
    private readonly extensionUri: vscode.Uri
  ) {}

  get recipeRepository(): RecipeRepository | undefined {
    return this._recipeRepository;
  }

  setRecipeRepository(repository: RecipeRepository): void {
    this._recipeRepository = repository;
  }

  get workspaceRoot(): string {
    return this.config.workspaceRoot;
  }

  get previewInFlight(): boolean {
    return this._previewInFlight;
  }

  setPreviewInFlight(value: boolean): void {
    this._previewInFlight = value;
  }

  get scaffoldHandler(): (() => Promise<void>) | undefined {
    return this._scaffoldHandler;
  }

  setScaffoldHandler(handler: () => Promise<void>): void {
    this._scaffoldHandler = handler;
  }

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
      this.state.currentRecipe = await this.ensureRecipeDetails(
        this.state.currentRecipe
      );
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

  setContextMatching(
    matches: readonly import('../../shared').ContextRecipeMatch[],
    slotsByRecipe: Record<string, string[]>
  ): void {
    this.state.setContextMatching(matches, slotsByRecipe);
    this.postState();
  }

  run(recipe: RecipeSchema, initialArgs: Record<string, string> = {}): void {
    void this.runInternal(recipe, initialArgs);
  }

  postState(): void {
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

  postMessage(message: unknown): void {
    void this.view?.webview.postMessage(message);
  }

  argsKey(args: Record<string, string>): string {
    const keys = Object.keys(args).sort();
    const ordered: Record<string, string> = {};
    for (const key of keys) {
      ordered[key] = args[key];
    }
    return JSON.stringify(ordered);
  }

  private async handleMessage(message: unknown): Promise<void> {
    if (!isWebviewToExtensionMessage(message)) return;
    await handleWebviewMessage(this, message);
  }

  private async runInternal(
    recipe: RecipeSchema,
    initialArgs: Record<string, string>
  ): Promise<void> {
    const hydrated = await this.ensureRecipeDetails(recipe);
    this.state.selectRecipe(hydrated, initialArgs);
    await this.revealAndPostState();
  }

  async ensureRecipeDetails(recipe: RecipeSchema): Promise<RecipeSchema> {
    if (recipe.templatesLoaded !== false) {
      return recipe;
    }
    try {
      return await this.bridge.describe(recipe.id);
    } catch {
      return recipe;
    }
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
}
