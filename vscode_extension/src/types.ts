export interface RecipeArg {
  name: string;
  abbr: string | null;
  help: string | null;
  required: boolean;
  defaultsTo: string | null;
  inputKind: 'text' | 'file' | 'directory' | 'enum' | 'dartType' | 'symbol';
  options: string[];
  allowCustomValue: boolean;
  contextKey: string | null;
}

export interface RecipeTemplatePreview {
  label: string;
  path: string;
  content: string;
}

export interface RecipeSchema {
  id: string;
  name: string;
  description: string;
  args: RecipeArg[];
  previewTemplates: RecipeTemplatePreview[];
}

export interface PatchInfo {
  index: number;
  offset: number;
  length: number;
  replacement: string;
  description: string | null;
}

export interface FilePreview {
  path: string;
  kind: 'edit' | 'create' | 'other';
  isNew: boolean;
  skipped: boolean;
  original: string;
  modified: string;
  preview?: string;
  patches: PatchInfo[];
}

export interface ListResponse {
  ok: boolean;
  error?: string;
  recipes?: RecipeSchema[];
}

export interface PreviewResponse {
  ok: boolean;
  error?: string;
  recipe?: string;
  files?: FilePreview[];
}

export interface ApplyResponse {
  ok: boolean;
  error?: string;
  recipe?: string;
  applied?: string[];
}

export interface FileSelection {
  include: boolean;
  patches?: number[];
}

export interface SelectionPayload {
  files: Record<string, FileSelection>;
}

export type HostCommand =
  | { command: 'list' }
  | { command: 'preview'; recipe: string; args: Record<string, string> }
  | {
      command: 'apply';
      recipe: string;
      args: Record<string, string>;
      selection: SelectionPayload;
    };
