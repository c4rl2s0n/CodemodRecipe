import type {
  BootstrapPhase,
  RunnerSessionPhase,
  RunnerTab,
  FilePreviewKind,
} from './constants';

/** JSON-native recipe argument value. */
export type ArgValue =
  | string
  | boolean
  | ArgValue[]
  | { [key: string]: ArgValue };

export type ArgValues = Record<string, ArgValue>;

/** Builtin kind name, registered type id, or a one-element list (`["ClassProperty"]`). */
export type ArgInputKindSpec = string | string[];


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
  inputKind: ArgInputKindSpec;
  options: string[];
  optionsFromMap?: string | null;
  allowCustomValue: boolean;
  /** @deprecated Prefer {@link from}. */
  contextKey: string | null;
  /** Builtin key, template, or tree-sitter query derivation. */
  from?: ArgFrom | null;
  /** Nested recipe ids that contribute this unbound arg (absent/empty when parent-declared). */
  fromRecipes?: string[];
  /** Resolved fields when `inputKind` is a named type or list of a named type. */
  fields?: RecipeArg[];
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
  args?: ArgValues;
}

export interface RecipeSchema {
  id: string;
  name: string;
  description: string;
  /** Workspace-relative path to the recipe Luau module. */
  sourceFile?: string | null;
  args: RecipeArg[];
  /** Explorer QuickPick opt-in entries (`kind` + optional path `if`). */
  explorerMenu?: ExplorerMenuEntry[] | null;
  /** Shell commands after successful apply (Jinja-rendered). */
  postExecution?: string[] | null;
  templatesLoaded?: boolean;
  previewTemplates?: { label: string; path: string; content?: string }[];
}

export interface TypeSchema {
  id: string;
  description?: string | null;
  fields: RecipeArg[];
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

export interface FilePreview {
  path: string;
  kind: FilePreviewKind;
  isNew: boolean;
  skipped: boolean;
  snippet?: string;
  original?: string;
  modified?: string;
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
  /** Number of maps loaded from `.codemod/maps/`. */
  mapsLoaded?: number;
  mapIds?: string[];
  varIds?: string[];
  typeIds?: string[];
  types?: TypeSchema[];
  languageIds?: string[];
  /** Registered query ids (`libName.queryName`). */
  queryIds?: string[];
  /** Registered query id → source string. */
  queries?: Record<string, string>;
  components?: ComponentSchema[];
  stack?: StackSchema;
  testSuites?: TestSuiteSchema[];
}

export interface ComponentSchema {
  id: string;
  description?: string;
  operations: string[];
  hasScaffold: boolean;
  isRoot: boolean;
  sourceFile?: string;
}

/** Catalog view of a `*.test.luau` suite (discovery for the Testing view). */
export interface TestSuiteSchema {
  id: string;
  description?: string;
  target?: string;
  /** Workspace-relative path to the suite module. */
  path?: string;
  /** Sorted test function names. */
  testNames: string[];
}

export interface TestCaseResult {
  name: string;
  ok: boolean;
  error?: string;
  /** 1-based line in the suite file (failures only). */
  line?: number;
}

export interface TestSuiteResult {
  id: string;
  path?: string;
  tests: TestCaseResult[];
}

export interface TestRunReport {
  ok: boolean;
  error?: string;
  suites: TestSuiteResult[];
  totals: { passed: number; failed: number };
}

export interface StackSchema {
  id: string;
  description?: string;
  components: string[];
  codebase?: string[];
  operations?: ComponentOperationSchema[];
  relations?: Record<string, RelationKindSchema>;
}

export interface RelationKindSchema {
  layout: string;
}

export interface ComponentOperationSchema {
  name: string;
  description?: string;
  args: RecipeArg[];
}

export interface ComponentDetailSchema {
  id: string;
  description?: string;
  hasIdentify: boolean;
  hasVerify?: boolean;
  hasRelate?: boolean;
  hasToSummary: boolean;
  hasScaffold: boolean;
  isRoot: boolean;
  sourceFile?: string;
  scaffoldArgs: RecipeArg[];
  operations: ComponentOperationSchema[];
}

export interface DescribeComponentResponse {
  ok: boolean;
  error?: string;
  component?: ComponentDetailSchema;
}

export interface ComponentInstance {
  id: string;
  componentId: string;
  locator: unknown;
  /** Parse-hook payload (AST projection). */
  ast: unknown;
  summary?: unknown;
}

export interface ComponentFinding {
  severity: string;
  code: string;
  message: string;
  instanceId?: string;
  locator?: unknown;
}

/** A resolved relation between two flat instances (or an external symbol).
 * Duplicates by (from, kind, target) merge into one canonical edge; `sources`
 * lists every distinct derivation (multiplicity = sources.length). */
export interface ComponentEdge {
  from: string;
  kind: string;
  to?: string;
  symbol?: string;
  sources: unknown[];
  /** Extra component-provided fields pass through (e.g. cardinality). */
  [extra: string]: unknown;
}

/** Kind-level aggregation entry: (fromComponentId, kind, toComponentId) + count. */
export interface KindEdge {
  from: string;
  kind: string;
  to: string;
  count: number;
  external?: boolean;
}

export interface ComponentScanResponse {
  ok: boolean;
  error?: string;
  instances?: ComponentInstance[];
  findings?: ComponentFinding[];
  /** Resolved relation edges over the scanned forest (relate phase). */
  edges?: ComponentEdge[];
  /** Kind-level aggregation of edges with counts. */
  kindGraph?: KindEdge[];
  diagnostics?: RecipeDiagnostic[];
}

export interface CoverageGap {
  from: number;
  to: number;
}

export interface CoverageTotals {
  ratio: number;
  coveredNonblank: number;
  gapNonblank: number;
}

export interface FileCoverage {
  path: string;
  ratio: number;
  coveredNonblank: number;
  gapNonblank: number;
  gaps?: CoverageGap[];
}

export interface ComponentCoverageResponse {
  ok: boolean;
  error?: string;
  skipped?: boolean;
  warning?: string;
  aggregate?: CoverageTotals;
  files?: FileCoverage[];
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

export interface TranscriptStats {
  addedLines?: number;
  removedLines?: number;
}

export interface TranscriptEvent {
  kind: string;
  detail: string;
  includedInPreview?: boolean;
  stats?: TranscriptStats;
  payload?: Record<string, unknown>;
  expandable?: boolean;
}

export interface PreviewResponse {
  ok: boolean;
  error?: string;
  recipe?: string;
  previewToken?: string;
  files?: FilePreview[];
  transcript?: TranscriptEvent[];
  /** c:log details — top-level for MCP agents. */
  logs?: string[];
  applyMayDiverge?: boolean;
  diagnostics?: RecipeDiagnostic[];
}

export interface DescribeResponse {
  ok: boolean;
  error?: string;
  recipe?: RecipeSchema;
}

export interface ComponentInvokeResponse {
  ok: boolean;
  error?: string;
  /** JSON return value from read hooks (identify/parse/verify/toSummary). */
  result?: unknown;
  instance?: ComponentInstance;
  findings?: ComponentFinding[];
  files?: FilePreview[];
  transcript?: TranscriptEvent[];
  /** c:log details — top-level for MCP agents (scan has no logs). */
  logs?: string[];
  applied?: string[];
}

export interface ApplyResponse {
  ok: boolean;
  error?: string;
  recipe?: string;
  applied?: string[];
  transcript?: TranscriptEvent[];
  /** c:log details — top-level for MCP agents. */
  logs?: string[];
  diagnostics?: RecipeDiagnostic[];
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
  | { command: 'diff'; recipe: string; args: ArgValues; path: string }
  | {
      command: 'preview';
      recipe: string;
      args: ArgValues;
      snippetLines?: number;
    }
  | {
      command: 'apply';
      recipe: string;
      args: ArgValues;
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
  | { command: 'generateAstPath'; path: string; offset: number }
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
    }
  | {
      command: 'queryCaptures';
      query?: string;
      queryId?: string;
    }
  | { command: 'componentScan' }
  | { command: 'componentCoverage' }
  | { command: 'componentGraph'; level?: string; out?: string }
  | { command: 'componentSnapshot'; path?: string }
  | {
      command: 'componentIncrementalScan';
      dirtyPaths?: string[];
      detectChanges?: boolean;
      force?: boolean;
    }
  | { command: 'componentListInstances' }
  | {
      command: 'componentInvokeOperation';
      instanceId: string;
      operation: string;
      args?: ArgValues;
      preview?: boolean;
    }
  | {
      command: 'componentScaffold';
      componentId: string;
      args?: ArgValues;
      preview?: boolean;
    }
  | {
      command: 'componentInvokeHook';
      componentId: string;
      hook: string;
      preview?: boolean;
      locator?: unknown;
      instance?: unknown;
      operation?: string;
      args?: ArgValues;
    }
  | {
      command: 'describeComponent';
      componentId: string;
    }
  | {
      command: 'stackInvokeOperation';
      operation: string;
      preview?: boolean;
      args?: ArgValues;
    }
  | {
      command: 'runTests';
      filter?: string;
    };

export interface QueryCapturesResponse {
  ok: boolean;
  error?: string;
  captures?: string[];
}

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
}

export interface SelectionPayload {
  files: Record<string, FileSelection>;
}

export interface ContextRecipeMatch {
  recipeId: string;
  name: string;
  description: string;
  args: ArgValues;
  complete: boolean;
}

/** Host-owned session snapshot pushed with every `state` message. */
export interface RunnerSessionSnapshot {
  id: number;
  phase: RunnerSessionPhase;
  activeRequestId?: number;
  previewStatus?: string;
  lastPreviewArgsKey?: string;
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
  initialArgs: ArgValues;
  activeTab: RunnerTab;
  autoPreviewDebounceMs?: number;
  /** Recipes matching local editor context (Tier A). */
  contextMatches?: readonly ContextRecipeMatch[];
  /** recipeId → slot ids assigned in settings. */
  slotsByRecipe?: Record<string, string[]>;
  /** Host session machine; authoritative for preview/apply/pick in-flight. */
  session: RunnerSessionSnapshot;
  /** Component kinds from host catalog. */
  components?: readonly ComponentSchema[];
  stack?: StackSchema;
  componentInstances?: readonly ComponentInstance[];
  componentFindings?: readonly ComponentFinding[];
  selectedComponentId?: string;
  selectedInstanceId?: string;
  componentDetail?: ComponentDetailSchema;
  componentAction?: string;
  componentArgs?: ArgValues;
  componentFiles?: readonly FilePreview[];
  componentTranscript?: readonly TranscriptEvent[];
  componentScanError?: string;
  componentOpError?: string;
  /** JSON result from the last component hook dry-run. */
  componentDryRunResult?: unknown;
  componentDryRunHook?: string;
  /** Latest componentCoverage result (undefined until first fetch). */
  componentCoverage?: ComponentCoverageResponse;
  /** Selected stack-level operation (mutually exclusive with componentAction). */
  stackAction?: string;
}

export interface PersistedWebviewState {
  recipeId?: string;
  activeTab: RunnerTab;
  argValues: ArgValues;
  files: FilePreview[];
  transcript?: TranscriptEvent[];
  applyMayDiverge?: boolean;
  activeChangeIndex: number;
  lastPreviewArgsKey: string;
  /** @deprecated Never restore as true across host restarts; kept for migration. */
  lastPreviewSuccess: boolean;
}

/** Discriminated result for ACK-required webview ↔ host RPCs. */
export type RpcResult<T = void> =
  | { status: 'ok'; value: T }
  | { status: 'error'; message: string }
  | { status: 'cancelled' };
