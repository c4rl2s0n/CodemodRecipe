import type { BootstrapPhase, RunnerTab, ArgInputKind, FilePreviewKind } from './constants';


export interface RecipeArg {
  name: string;
  abbr: string | null;
  help: string | null;
  required: boolean;
  defaultsTo: string | null;
  inputKind: ArgInputKind;
  options: string[];
  allowCustomValue: boolean;
  contextKey: string | null;
}

export interface RecipeSchema {
  id: string;
  name: string;
  description: string;
  args: RecipeArg[];
  templatesLoaded?: boolean;
  previewTemplates?: { label: string; path: string; content?: string }[];
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
}

export interface RecipeCatalogResponse {
  ok: boolean;
  error?: string;
  recipes?: RecipeSchema[];
  diagnostics?: RecipeDiagnostic[];
}

/** @deprecated Use {@link RecipeCatalogResponse} */
export type ListResponse = RecipeCatalogResponse;

/** @deprecated Use {@link RecipeCatalogResponse} */
export type ReloadResponse = RecipeCatalogResponse;

export interface PreviewResponse {
  ok: boolean;
  error?: string;
  recipe?: string;
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

export type HostCommand =
  | { command: 'list' }
  | { command: 'reload' }
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
      selection: SelectionPayload;
    }
  | { command: 'generateAstPath'; path: string; offset: number };







export interface FileSelection {
  include: boolean;
  patches?: number[];
}

export interface SelectionPayload {
  files: Record<string, FileSelection>;
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

export interface AstPathResult {
  ok: boolean;
  error?: string;
  path?: {
    navigate: Array<{
      kind?: string;
      name: string;
      match?: string;
    }>;
    anchor: string;
    offset: number;
  };
}
