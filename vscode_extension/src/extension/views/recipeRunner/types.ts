import type { ExtensionConfig } from '../../config/extensionConfig';
import type { DiffContentProvider } from '../../diff/diffContentProvider';
import type { HostBridge } from '../../host/hostBridge';
import type { ArgValues } from '../../../shared';
import type { RecipeRunnerState } from '../recipeRunnerState';
import type { RecipeRepository } from '../../recipes/recipeRepository';

export interface RecipeRunnerHandlerHost {
  readonly workspaceRoot: string;
  readonly state: RecipeRunnerState;
  readonly bridge: HostBridge;
  readonly config: ExtensionConfig;
  readonly diffProvider: DiffContentProvider;
  get previewInFlight(): boolean;
  setPreviewInFlight(value: boolean): void;
  /** Monotonic generation; bumped when a preview is superseded or finished. */
  get previewGeneration(): number;
  beginPreviewGeneration(): number;
  get scaffoldHandler(): (() => Promise<void>) | undefined;
  postState(): void;
  postMessage(message: unknown): void;
  run(
    recipe: import('../../../shared').RecipeSchema,
    initialArgs?: ArgValues
  ): void;
  ensureRecipeDetails(
    recipe: import('../../../shared').RecipeSchema
  ): Promise<import('../../../shared').RecipeSchema>;
  argsKey(args: ArgValues): string;
  get recipeRepository(): RecipeRepository | undefined;
  /** Reset host preview flag + bump session (webview recreate / ready). */
  resetSession(): void;
  /** Publish runtime diagnostics (Problems panel). */
  publishRuntimeDiagnostics?(diagnostics: readonly import('../../../shared').RecipeDiagnostic[]): void;
  scanComponents?(): Promise<void>;
  refreshComponentCoverage?(): Promise<void>;
  openCoverageGaps?(
    relativePath: string,
    gaps: ReadonlyArray<{ from: number; to: number }>
  ): Promise<void>;
  clearCoverageGaps?(): void;
}

