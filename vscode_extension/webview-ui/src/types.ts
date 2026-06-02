export type RunnerTab = 'recipes' | 'runner';

export type ArgInputKind =
  | 'text'
  | 'file'
  | 'directory'
  | 'enum'
  | 'dartType'
  | 'symbol';

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
  kind: 'edit' | 'create' | 'other';
  isNew: boolean;
  skipped: boolean;
  snippet?: string;
  original?: string;
  modified?: string;
  preview?: string;
  patches: PatchInfo[];
}

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
  recipe?: RecipeSchema;
  initialArgs: Record<string, string>;
  activeTab: RunnerTab;
  autoPreviewDebounceMs: number;
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
