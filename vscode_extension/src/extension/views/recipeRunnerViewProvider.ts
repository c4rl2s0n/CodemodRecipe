import * as vscode from 'vscode';
import { EXTENSION, VIEWS } from '../constants';
import { DiffContentProvider } from '../diff/diffContentProvider';
import { ExtensionConfig } from '../config/extensionConfig';
import { HostBridge } from '../host/hostBridge';
import {
  EXTENSION_TO_WEBVIEW,
  argsKey as fingerprintArgs,
  isWebviewToExtensionMessage,
  type ArgValues,
  type RecipeDiagnostic,
  type RecipeSchema,
} from '../../shared';
import { renderRecipeViewHtml } from '../webview/recipeViewHtml';
import { RecipeRunnerState } from './recipeRunnerState';
import {
  handleWebviewMessage,
  type RecipeRunnerHandlerHost,
} from './recipeRunner';
import type { RecipeRepository } from '../recipes/recipeRepository';
import { CoverageDecorations } from '../coverage/coverageDecorations';

export class RecipeRunnerViewProvider
  implements vscode.WebviewViewProvider, RecipeRunnerHandlerHost
{
  private view: vscode.WebviewView | undefined;
  private renderPending = false;
  private webviewHtmlLoaded = false;
  private messageSub: vscode.Disposable | undefined;
  private _previewInFlight = false;
  private _previewGeneration = 0;
  /** Serialize webview messages so preview/pick cannot overlap silently. */
  private messageChain: Promise<void> = Promise.resolve();
  readonly state = new RecipeRunnerState();
  private readonly coverageDecorations = new CoverageDecorations();
  private _scaffoldHandler: (() => Promise<void>) | undefined;
  private _recipeRepository: RecipeRepository | undefined;
  private _runtimeDiagnostics: ((diagnostics: readonly RecipeDiagnostic[]) => void) | undefined;

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

  setRuntimeDiagnosticsPublisher(
    publish: (diagnostics: readonly RecipeDiagnostic[]) => void
  ): void {
    this._runtimeDiagnostics = publish;
  }

  publishRuntimeDiagnostics(diagnostics: readonly RecipeDiagnostic[]): void {
    this._runtimeDiagnostics?.(diagnostics);
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

  get previewGeneration(): number {
    return this._previewGeneration;
  }

  beginPreviewGeneration(): number {
    this._previewGeneration += 1;
    return this._previewGeneration;
  }

  get scaffoldHandler(): (() => Promise<void>) | undefined {
    return this._scaffoldHandler;
  }

  setScaffoldHandler(handler: () => Promise<void>): void {
    this._scaffoldHandler = handler;
  }

  resetSession(): void {
    this._previewInFlight = false;
    this._previewGeneration += 1;
    this.bridge.cancelInFlight('Session reset');
    this.state.beginSession();
  }

  resolveWebviewView(webviewView: vscode.WebviewView): void {
    this.messageSub?.dispose();
    this.view = webviewView;
    // Moving between side bars (or reactivation) recreates the WebviewView;
    // always re-inject HTML for the new host and clear stale in-flight flags.
    this.webviewHtmlLoaded = false;
    this.resetSession();
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
    this.messageSub = webviewView.webview.onDidReceiveMessage(
      (message: unknown) => {
        this.enqueueMessage(message);
      }
    );
    webviewView.onDidDispose(() => {
      if (this.view !== webviewView) {
        return;
      }
      this.view = undefined;
      this.webviewHtmlLoaded = false;
      this._previewInFlight = false;
      this.messageSub?.dispose();
      this.messageSub = undefined;
    });
    webviewView.onDidChangeVisibility(() => {
      if (this.view !== webviewView || !webviewView.visible) {
        return;
      }
      // Secondary side bar reactivation can leave an empty body; re-inject.
      this.ensureWebviewHtml();
      this.postState();
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

  setComponentCatalog(
    components: readonly import('../../shared').ComponentSchema[],
    stack?: import('../../shared').StackSchema
  ): void {
    this.state.setComponentCatalog(components, stack);
    this.postState();
  }

  clearComponentScan(): void {
    this.state.setComponentScan([], [], undefined);
    this.state.setComponentCoverage(undefined);
    this.coverageDecorations.clear();
    this.postState();
  }

  async scanComponents(): Promise<void> {
    try {
      const response = await this.bridge.componentScan();
      if (!response.ok) {
        this.state.setComponentScan(
          [],
          [],
          response.error ?? 'component scan failed'
        );
        this.state.setComponentCoverage(undefined);
      } else {
        this.state.setComponentScan(
          response.instances ?? [],
          response.findings ?? []
        );
        await this.refreshComponentCoverage();
        return;
      }
    } catch (err) {
      this.state.setComponentScan(
        [],
        [],
        err instanceof Error ? err.message : String(err)
      );
      this.state.setComponentCoverage(undefined);
    }
    this.postState();
  }

  async incrementalComponentScan(dirtyPaths: string[]): Promise<void> {
    try {
      const response = await this.bridge.componentIncrementalScan({
        dirtyPaths,
        detectChanges: false,
      });
      if (!response.ok) {
        this.state.setComponentScan(
          this.state.componentInstances,
          this.state.componentFindings,
          response.error ?? 'incremental scan failed'
        );
      } else {
        this.state.setComponentScan(
          response.instances ?? [],
          response.findings ?? []
        );
        await this.refreshComponentCoverage();
        return;
      }
    } catch (err) {
      this.state.componentScanError =
        err instanceof Error ? err.message : String(err);
    }
    this.postState();
  }

  async refreshComponentCoverage(): Promise<void> {
    try {
      const coverage = await this.bridge.componentCoverage();
      this.state.setComponentCoverage(coverage);
    } catch (err) {
      this.state.setComponentCoverage({
        ok: false,
        error: err instanceof Error ? err.message : String(err),
      });
    }
    this.postState();
  }

  async openCoverageGaps(
    relativePath: string,
    gaps: ReadonlyArray<{ from: number; to: number }>
  ): Promise<void> {
    await this.coverageDecorations.openWithGaps(
      this.workspaceRoot,
      relativePath,
      gaps
    );
  }

  clearCoverageGaps(): void {
    this.coverageDecorations.clear();
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

  run(recipe: RecipeSchema, initialArgs: ArgValues = {}): void {
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

  argsKey(args: ArgValues): string {
    return fingerprintArgs(args);
  }

  private enqueueMessage(message: unknown): void {
    this.messageChain = this.messageChain
      .then(() => this.handleMessage(message))
      .catch(() => {
        // Keep the chain alive after handler failures.
      });
  }

  private async handleMessage(message: unknown): Promise<void> {
    if (!isWebviewToExtensionMessage(message)) return;
    await handleWebviewMessage(this, message);
  }

  private async runInternal(
    recipe: RecipeSchema,
    initialArgs: ArgValues
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
    if (!this.view) {
      return;
    }
    // VS Code can clear the webview body (e.g. secondary side bar) without
    // disposing; treat empty html as needing a fresh inject.
    if (this.webviewHtmlLoaded && this.view.webview.html) {
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
