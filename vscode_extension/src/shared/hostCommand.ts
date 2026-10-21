import type { ArgValues, SelectionPayload } from './types';

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
      /** Required when preview is false (apply). */
      previewToken?: string;
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
      /** Required when preview is false (apply). */
      previewToken?: string;
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

