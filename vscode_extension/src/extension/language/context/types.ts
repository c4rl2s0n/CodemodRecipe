import type { HostMethodSpec } from '../hostApiSurface';

export type TextDocument = {
  lineCount: number;
  lineAt(line: number): { text: string };
  getText(): string;
};

export type RecipeIdContext = {
  typed: string;
  /** Character offset of the typed fragment on the line. */
  start: number;
  kind: 'id' | 'run' | 'map' | 'var' | 'queryLib';
};

export type MethodSpecContext = {
  method: string;
  spec: HostMethodSpec;
  /** Keys already present in the open `{ … }` table. */
  setKeys: Set<string>;
  /** Partial key being typed (may be empty). */
  typedKey: string;
  /** Character start of typedKey on the current line. */
  keyStart: number;
  /** True when completing a string value after `key = "`. */
  valueField?: string;
  typedValue?: string;
  valueStart?: number;
};
