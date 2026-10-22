export type { HostCommand } from './hostCommand.g';



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

