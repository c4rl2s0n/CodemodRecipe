import type { BootstrapPhase, RunnerTab, ArgInputKind, FilePreviewKind } from './constants';


/** How to derive an arg from editor context (string builtin or structured spec). */
export type ArgFrom =
  | string
  | {
      template?: string;
      query?: string | string[];
      capture?: string;
      extract?: 'text' | 'kind' | 'exists' | 'count';
      scope?: 'enclosing' | 'selection' | 'first';
      language?: string;
      as?: string;
      onNoMatch?: 'omit' | 'empty';
    };

export interface RecipeArg {
  name: string;
  abbr: string | null;
  help: string | null;
  required: boolean;
  defaultsTo: string | null;
  inputKind: ArgInputKind;
  options: string[];
  allowCustomValue: boolean;
  /** @deprecated Prefer {@link from}. */
  contextKey: string | null;
  /** Builtin key, template, or tree-sitter query derivation. */
  from?: ArgFrom | null;
  /** Nested recipe ids that contribute this unbound arg (absent/empty when parent-declared). */
  fromRecipes?: string[];
}

export interface DeriveArgsRequest {
  recipe: string;
  source: string;
  language?: string;
  path?: string;
  cursorOffset: number;
  selectionStart: number;
  selectionEnd: number;
  context: Record<string, string>;
}

export interface DeriveArgsResponse {
  ok: boolean;
  error?: string;
  args?: Record<string, string>;
}

export interface RecipeSchema {
  id: string;
  name: string;
  description: string;
  /** Workspace-relative path to the recipe YAML file. */
  sourceFile?: string | null;
  args: RecipeArg[];
  /** Explorer QuickPick opt-in entries (`kind` + optional path `if`). */
  explorerMenu?: ExplorerMenuEntry[] | null;
  templatesLoaded?: boolean;
  previewTemplates?: { label: string; path: string; content?: string }[];
}

export interface ExplorerMenuEntry {
  kind: 'file' | 'folder';
  if?: string | null;
  /** Arg name → MiniJinja expression over click `path` (unevaluated catalog form). */
  args?: Record<string, string>;
}

export interface ExplorerRecipeMatch {
  recipeId: string;
  args: Record<string, string>;
}

export interface FilterExplorerRecipesRequest {
  path: string;
  kind: 'file' | 'folder';
}

export interface FilterExplorerRecipesResponse {
  ok: boolean;
  error?: string;
  matches?: ExplorerRecipeMatch[];
}

export interface PatchInfo {
  index: number;
  offset: number;
  length: number;
  replacement?: string;
  replacementPreview?: string;
  description: string | null;
}

export interface FilePreview {
  path: string;
  kind: FilePreviewKind;
  isNew: boolean;
  skipped: boolean;
  snippet?: string;
  original?: string;
  modified?: string;
  preview?: string;
  patches: PatchInfo[];
}

export interface RecipeDiagnostic {
  severity: 'error' | 'warning';
  code: string;
  message: string;
  sources?: { file: string; line?: number; column?: number }[];
  hint?: string;
  relatedRecipe?: string;
}

export interface RecipeCatalogResponse {
  ok: boolean;
  error?: string;
  recipes?: RecipeSchema[];
  diagnostics?: RecipeDiagnostic[];
  /** Number of YAML maps loaded from `.codemod/maps/`. */
  mapsLoaded?: number;
  mapIds?: string[];
  varIds?: string[];
  languageIds?: string[];
}

export interface BootstrapResponse {
  ok: boolean;
  error?: string;
  edit_policy?: string;
  companions?: string[];
  written?: string[];
  skipped?: string[];
}

/** @deprecated Use {@link RecipeCatalogResponse} */
export type ListResponse = RecipeCatalogResponse;

/** @deprecated Use {@link RecipeCatalogResponse} */
export type ReloadResponse = RecipeCatalogResponse;

export interface PreviewResponse {
  ok: boolean;
  error?: string;
  recipe?: string;
  previewToken?: string;
  files?: FilePreview[];
}

export interface DescribeResponse {
  ok: boolean;
  error?: string;
  recipe?: RecipeSchema;
}

export interface ApplyResponse {
  ok: boolean;
  error?: string;
  recipe?: string;
  applied?: string[];
}

export interface DiffResponse {
  ok: boolean;
  error?: string;
  recipe?: string;
  file?: FilePreview;
}

export interface ValidateResponse {
  ok: boolean;
  error?: string;
  diagnostics?: RecipeDiagnostic[];
}

export type HostCommand =
  | { command: 'list' }
  | { command: 'reload' }
  | { command: 'validate'; recipe?: string }
  | { command: 'describe'; recipe: string }
  | { command: 'diff'; recipe: string; args: Record<string, string>; path: string }
  | {
      command: 'preview';
      recipe: string;
      args: Record<string, string>;
      snippetLines?: number;
    }
  | {
      command: 'apply';
      recipe: string;
      args: Record<string, string>;
      previewToken: string;
      selection: SelectionPayload;
    }
  | {
      command: 'deriveArgs';
      recipe: string;
      source: string;
      language?: string;
      path?: string;
      cursorOffset: number;
      selectionStart: number;
      selectionEnd: number;
      context: Record<string, string>;
    }
  | {
      command: 'filterExplorerRecipes';
      path: string;
      kind: 'file' | 'folder';
    }
  | {
      command: 'bootstrap';
      force?: boolean;
      editPolicy?: string;
      companions?: string[];
    }
  | {
      command: 'dumpAst';
      path?: string;
      source?: string;
      language?: string;
      namedOnly?: boolean;
    }
  | {
      command: 'debugQuery';
      path?: string;
      source?: string;
      language?: string;
      query: string;
      instrument?: boolean;
      includeSexp?: boolean;
    }
  | {
      command: 'generateQuery';
      path?: string;
      source?: string;
      language?: string;
      start: number;
      end?: number;
      includeTextPredicates?: boolean;
      captureLeaf?: string;
      maxDepth?: number;
    }
  | {
      command: 'resolveStaticPath';
      template: string;
    };

export interface AstNodeDto {
  kind: string;
  named: boolean;
  field?: string;
  start: { byte: number; line: number; column: number };
  end: { byte: number; line: number; column: number };
  isError: boolean;
  isMissing: boolean;
  text?: string;
  children: AstNodeDto[];
}

export interface DumpAstResponse {
  ok: boolean;
  error?: string;
  hasError?: boolean;
  root?: AstNodeDto;
}

export interface CaptureInfoDto {
  name: string;
  kind: string;
  start: number;
  end: number;
  startLine: number;
  startColumn: number;
  endLine: number;
  endColumn: number;
  text?: string;
  depth: number;
  isLayer: boolean;
  queryStart?: number;
  queryEnd?: number;
}

export interface DebugMatchDto {
  root: {
    kind: string;
    start: number;
    end: number;
    startLine: number;
    startColumn: number;
    endLine: number;
    endColumn: number;
    text?: string;
  };
  captures: CaptureInfoDto[];
}

export interface DebugQueryResultDto {
  hasError: boolean;
  matchCount: number;
  matches: DebugMatchDto[];
  instrumentedQuery?: string;
  rootSexp?: string;
}

export interface DebugQueryResponse {
  ok: boolean;
  error?: string;
  result?: DebugQueryResultDto;
}

export interface GenerateQueryResponse {
  ok: boolean;
  error?: string;
  query?: string;
  captureSuggestion?: string;
}

export interface ResolveStaticPathResponse {
  ok: boolean;
  error?: string;
  path?: string;
  staticResolvable: boolean;
}


export interface FileSelection {
  include: boolean;
  patches?: number[];
}

export interface SelectionPayload {
  files: Record<string, FileSelection>;
}

export interface ContextRecipeMatch {
  recipeId: string;
  name: string;
  description: string;
  args: Record<string, string>;
  complete: boolean;
}

export interface RecipeViewState {
  recipes: readonly RecipeSchema[];
  discoveryError?: string;
  diagnostics: readonly RecipeDiagnostic[];
  recipesRefreshing: boolean;
  bootstrapInFlight: boolean;
  bootstrapPhase: BootstrapPhase;
  bootstrapError?: string;
  recipe?: RecipeSchema;
  initialArgs: Record<string, string>;
  activeTab: RunnerTab;
  autoPreviewDebounceMs?: number;
  /** Recipes matching local editor context (Tier A). */
  contextMatches?: readonly ContextRecipeMatch[];
  /** recipeId → slot ids assigned in settings. */
  slotsByRecipe?: Record<string, string[]>;
}

export interface PersistedWebviewState {
  recipeId?: string;
  activeTab: RunnerTab;
  argValues: Record<string, string>;
  files: FilePreview[];
  activeChangeIndex: number;
  lastPreviewArgsKey: string;
  lastPreviewSuccess: boolean;
}
