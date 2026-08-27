/**
 * Luau recipe document context helpers (brace-aware, no full parser).
 */

import {
  hostMethodByName,
  type HostMethodSpec,
} from './hostApiSurface';

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

const ID_CHARS = String.raw`[\w./-]`;

/**
 * Detect partial string after `id =`, `c:run(`, `c:map(`, `c:var(`, `c:queryLib(`.
 */
export function recipeIdCompletionContext(before: string): RecipeIdContext | undefined {
  const patterns: Array<{ re: RegExp; kind: RecipeIdContext['kind'] }> = [
    { re: new RegExp(String.raw`\bc:run\s*\(\s*['"](${ID_CHARS}*)$`), kind: 'run' },
    { re: new RegExp(String.raw`\bc:map\s*\(\s*['"](${ID_CHARS}*)$`), kind: 'map' },
    { re: new RegExp(String.raw`\bc:var\s*\(\s*['"](${ID_CHARS}*)$`), kind: 'var' },
    {
      re: new RegExp(String.raw`\bc:queryLib\s*\(\s*['"](${ID_CHARS}*)$`),
      kind: 'queryLib',
    },
    {
      re: new RegExp(String.raw`(?:^|[\s,{])id\s*=\s*['"](${ID_CHARS}*)$`),
      kind: 'id',
    },
  ];
  for (const { re, kind } of patterns) {
    const match = re.exec(before);
    if (!match || match.index === undefined) {
      continue;
    }
    const typed = match[1] ?? '';
    const start = match.index + match[0].length - typed.length;
    return { typed, start, kind };
  }
  return undefined;
}

/** Recipe / map / var / queryLib id under the cursor (for hover / definition). */
export function matchIdReference(
  line: string,
  character: number
): { id: string; kind: RecipeIdContext['kind'] } | undefined {
  const patterns: Array<{ re: RegExp; kind: RecipeIdContext['kind'] }> = [
    { re: /\bc:run\s*\(\s*['"]([A-Za-z_][\w./-]*)['"]/, kind: 'run' },
    { re: /\bc:map\s*\(\s*['"]([A-Za-z_][\w./-]*)['"]/, kind: 'map' },
    { re: /\bc:var\s*\(\s*['"]([A-Za-z_][\w./-]*)['"]/, kind: 'var' },
    { re: /\bc:queryLib\s*\(\s*['"]([A-Za-z_][\w./-]*)['"]/, kind: 'queryLib' },
    { re: /(?:^|[\s,{])id\s*=\s*['"]([A-Za-z_][\w./-]*)['"]/, kind: 'id' },
  ];
  for (const { re, kind } of patterns) {
    const match = re.exec(line);
    if (!match || match.index === undefined) {
      continue;
    }
    const id = match[1];
    const start = match.index + match[0].indexOf(id);
    const end = start + id.length;
    if (character >= start && character <= end) {
      return { id, kind };
    }
  }
  return undefined;
}

/** `c:method` name under cursor. */
export function matchHostMethod(
  line: string,
  character: number
): string | undefined {
  const re = /\bc:([A-Za-z_]\w*)/g;
  for (const match of line.matchAll(re)) {
    const name = match[1];
    const start = match.index! + 2; // after `c:`
    const end = start + name.length;
    if (character >= start && character <= end) {
      return name;
    }
  }
  return undefined;
}

/** Template path under `path =` / `c:template({ path })` / long-string template refs. */
export function matchTemplatePath(
  line: string,
  character: number
): string | undefined {
  const match =
    /(?:path|template)\s*=\s*['"]([^'"]+\.template)['"]/.exec(line) ??
    /template\s*\(\s*\{\s*path\s*=\s*['"]([^'"]+)['"]/.exec(line);
  if (!match || match.index === undefined) {
    return undefined;
  }
  const path = match[1];
  const start = match.index + match[0].indexOf(path);
  const end = start + path.length;
  if (character >= start && character <= end) {
    return path;
  }
  return undefined;
}

export function documentTopLevelId(document: TextDocument): string | undefined {
  const text = document.getText();
  // Prefer first top-level `id = "..."` near the return table.
  const match = text.match(/(?:^|\n)\s*id\s*=\s*['"]([A-Za-z_][\w./-]*)['"]/);
  return match?.[1];
}

/** Line index of the top-level `id =` assignment, if within first 80 lines. */
export function documentTopLevelIdLine(
  document: TextDocument
): number | undefined {
  const limit = Math.min(document.lineCount, 80);
  for (let i = 0; i < limit; i++) {
    const text = document.lineAt(i).text;
    if (/^\s*id\s*=\s*['"][A-Za-z_][\w./-]*['"]/.test(text)) {
      return i;
    }
  }
  return undefined;
}

/**
 * Arg names from the recipe module `args = { { name = "…" }, … }` table.
 */
export function parseRecipeArgNames(source: string): string[] {
  const names: string[] = [];
  const seen = new Set<string>();
  // Prefer the args = { … } block before apply =
  const argsMatch = source.match(
    /\bargs\s*=\s*\{([\s\S]*?)\}\s*,?\s*(?:\n\s*(?:apply|postExecution|maps|types|queries|explorerMenu)\s*=|\n\s*\})/
  );
  const block = argsMatch?.[1] ?? source;
  const re = /\bname\s*=\s*['"]([A-Za-z_]\w*)['"]/g;
  let match: RegExpExecArray | null;
  while ((match = re.exec(block)) !== null) {
    const name = match[1];
    if (!seen.has(name)) {
      seen.add(name);
      names.push(name);
    }
  }
  return names;
}

/** `c.args.` completion prefix. */
export function argsFieldCompletionContext(
  before: string
): { typed: string; start: number } | undefined {
  const match = /\bc\.args\.([A-Za-z_]\w*)$/.exec(before);
  if (!match || match.index === undefined) {
    return undefined;
  }
  const typed = match[1] ?? '';
  const start = match.index + match[0].length - typed.length;
  return { typed, start };
}

/** `c:` method-name completion (before opening paren). */
export function hostMethodCompletionContext(
  before: string
): { typed: string; start: number } | undefined {
  const match = /\bc:([A-Za-z_]\w*)$/.exec(before);
  if (!match || match.index === undefined) {
    return undefined;
  }
  const typed = match[1] ?? '';
  const start = match.index + 2;
  return { typed, start };
}

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

/**
 * Brace-aware: find the nearest enclosing `c:method({` above the cursor and
 * collect sibling keys already set in that table.
 */
export function methodSpecContext(
  document: TextDocument,
  line: number,
  character: number
): MethodSpecContext | undefined {
  const beforeCursor = document.lineAt(line).text.slice(0, character);

  // Value completion: `language = "da` / `anchor = "st`
  const valueMatch =
    /^\s*([A-Za-z_]\w*)\s*=\s*['"]([\w./-]*)$/.exec(beforeCursor) ??
    /(?:^|[\s,{])([A-Za-z_]\w*)\s*=\s*['"]([\w./-]*)$/.exec(beforeCursor);
  let valueField: string | undefined;
  let typedValue: string | undefined;
  let valueStart: number | undefined;
  if (valueMatch && valueMatch.index !== undefined) {
    valueField = valueMatch[1];
    typedValue = valueMatch[2] ?? '';
    valueStart =
      valueMatch.index + valueMatch[0].length - (typedValue?.length ?? 0);
  }

  // Key completion: `  pat` or `  path =`
  const keyMatch = /^\s*([A-Za-z_]\w*)$/.exec(beforeCursor);
  const typedKey = keyMatch?.[1] ?? '';
  const keyStart = keyMatch
    ? beforeCursor.length - typedKey.length
    : beforeCursor.match(/^(\s*)/)?.[1].length ?? 0;

  const open = findEnclosingMethodCall(document, line, character);
  if (!open) {
    return undefined;
  }
  const spec = hostMethodByName(open.method);
  if (!spec || spec.keys.length === 0) {
    return undefined;
  }
  const setKeys = collectTableKeys(document, open.openLine, open.openCol, line);
  return {
    method: open.method,
    spec,
    setKeys,
    typedKey: valueField ? '' : typedKey,
    keyStart,
    valueField,
    typedValue,
    valueStart,
  };
}

function findEnclosingMethodCall(
  document: TextDocument,
  line: number,
  character: number
): { method: string; openLine: number; openCol: number } | undefined {
  // Scan backward counting braces; when depth returns to the call that opened
  // at `c:method({`, return that method.
  let depth = 0;
  let started = false;
  for (let i = line; i >= 0; i--) {
    const text = document.lineAt(i).text;
    const end = i === line ? character : text.length;
    for (let j = end - 1; j >= 0; j--) {
      const ch = text[j];
      if (ch === '}') {
        depth++;
        started = true;
      } else if (ch === '{') {
        if (depth === 0) {
          // Look for c:method( immediately before this `{`
          const prefix = text.slice(0, j);
          const m = /\bc:([A-Za-z_]\w*)\s*\(\s*$/.exec(prefix);
          if (m) {
            return { method: m[1], openLine: i, openCol: j };
          }
          // Also allow multi-line: `{` alone with `c:method(` on previous lines
          if (prefix.trim() === '' || /^\s*$/.test(prefix)) {
            for (let k = i; k >= Math.max(0, i - 3); k--) {
              const prev =
                k === i ? prefix : document.lineAt(k).text;
              const mm = /\bc:([A-Za-z_]\w*)\s*\(\s*\{?\s*$/.exec(
                k === i ? prev : prev.trimEnd()
              );
              if (mm) {
                return { method: mm[1], openLine: i, openCol: j };
              }
              const callOnly = /\bc:([A-Za-z_]\w*)\s*\(\s*$/.exec(
                document.lineAt(k).text
              );
              if (callOnly && k < i) {
                return { method: callOnly[1], openLine: i, openCol: j };
              }
            }
          }
          return undefined;
        }
        depth--;
        started = true;
      }
    }
    if (started && depth === 0 && i < line) {
      // Closed table above — stop if we never found a method (shouldn't happen).
    }
  }
  return undefined;
}

/**
 * Collect `key =` names inside a table from `openLine`/`openCol` up to `untilLine`.
 */
export function collectTableKeys(
  document: TextDocument,
  openLine: number,
  openCol: number,
  untilLine: number
): Set<string> {
  const keys = new Set<string>();
  let depth = 0;
  for (let i = openLine; i <= untilLine && i < document.lineCount; i++) {
    const text = document.lineAt(i).text;
    const start = i === openLine ? openCol + 1 : 0;
    const end = text.length;
    let j = start;
    while (j < end) {
      const ch = text[j];
      if (ch === '{') {
        depth++;
        j++;
        continue;
      }
      if (ch === '}') {
        depth--;
        j++;
        continue;
      }
      if (depth === 0) {
        const slice = text.slice(j);
        const km = /^([A-Za-z_]\w*)\s*=/.exec(slice);
        if (km) {
          keys.add(km[1]);
          j += km[0].length;
          continue;
        }
      }
      j++;
    }
  }
  return keys;
}

/**
 * Find nearby `c:run("id"` above the cursor for nested arg-table completion.
 */
export function findNearbyRunRecipeId(
  document: TextDocument,
  line: number
): string | undefined {
  for (let i = line; i >= Math.max(0, line - 30); i--) {
    const text = document.lineAt(i).text;
    const m = /\bc:run\s*\(\s*['"]([A-Za-z_][\w./-]*)['"]/.exec(text);
    if (m) {
      return m[1];
    }
  }
  return undefined;
}

/**
 * True when cursor is inside the second arg table of `c:run("id", { … })`.
 */
export function inRunArgsTable(
  document: TextDocument,
  line: number,
  character: number
): boolean {
  let runLine = -1;
  for (let i = line; i >= Math.max(0, line - 30); i--) {
    if (/\bc:run\s*\(\s*['"][A-Za-z_][\w./-]*['"]/.test(document.lineAt(i).text)) {
      runLine = i;
      break;
    }
  }
  if (runLine < 0) {
    return false;
  }

  let depth = 0;
  let seenOpen = false;
  for (let i = runLine; i <= line; i++) {
    const text = document.lineAt(i).text;
    const callAt = text.search(/\bc:run\s*\(/);
    const start = i === runLine && callAt >= 0 ? callAt : 0;
    const end = i === line ? character : text.length;
    for (let j = start; j < end; j++) {
      if (text[j] === '{') {
        depth++;
        seenOpen = true;
      } else if (text[j] === '}') {
        depth--;
      }
    }
  }
  return seenOpen && depth > 0;
}

export function collectSetRunArgKeys(
  document: TextDocument,
  line: number
): Set<string> {
  const keys = new Set<string>();
  for (let i = Math.max(0, line - 40); i <= line && i < document.lineCount; i++) {
    const text = document.lineAt(i).text;
    const m = /^\s*([A-Za-z_]\w*)\s*=/.exec(text);
    if (m && !['path', 'query', 'capture', 'text', 'language', 'anchor'].includes(m[1])) {
      // Prefer keys inside run args — also accept common ones
      keys.add(m[1]);
    }
    // More precise: any key = in the region after c:run
    const any = text.matchAll(/(?:^|[\s,{])([A-Za-z_]\w*)\s*=/g);
    for (const match of any) {
      keys.add(match[1]);
    }
  }
  return keys;
}

/** First `query =` line inside apply (for Query Tools CodeLens). */
export function firstQueryAssignLine(
  document: TextDocument
): number | undefined {
  for (let i = 0; i < document.lineCount; i++) {
    if (/\bquery\s*=/.test(document.lineAt(i).text)) {
      return i;
    }
  }
  return undefined;
}

/**
 * Resolve sibling `query =` in the same table as a `capture =` line (Luau).
 */
export function findSiblingQueryAssign(
  document: TextDocument,
  captureLine: number
): { line: number; rest: string } | undefined {
  const captureIndent =
    document.lineAt(captureLine).text.match(/^(\s*)/)?.[1].length ?? 0;
  for (let i = captureLine - 1; i >= Math.max(0, captureLine - 40); i--) {
    const text = document.lineAt(i).text;
    if (text.trim() === '') {
      continue;
    }
    const ind = text.match(/^(\s*)/)?.[1].length ?? 0;
    if (ind < captureIndent - 2 && text.includes('{')) {
      // went above table — keep scanning a bit
    }
    const m = text.match(/^\s*query\s*=\s*(.*)$/);
    if (m && ind <= captureIndent + 2) {
      return { line: i, rest: m[1] ?? '' };
    }
    if (/\bc:[A-Za-z_]\w*\s*\(/.test(text) && i < captureLine - 1) {
      break;
    }
  }
  return undefined;
}

/**
 * Read a Luau long string `[[...]]` starting at `line` after `query = [[`.
 */
export function readLuauLongString(
  document: TextDocument,
  startLine: number,
  rest: string
): string | undefined {
  const open = rest.match(/^\[(=*)\[(.*)$/);
  if (!open) {
    return undefined;
  }
  const eqs = open[1];
  const close = `]${eqs}]`;
  const first = open[2] ?? '';
  if (first.includes(close)) {
    return first.slice(0, first.indexOf(close));
  }
  const lines: string[] = [first];
  for (let i = startLine + 1; i < document.lineCount; i++) {
    const text = document.lineAt(i).text;
    const idx = text.indexOf(close);
    if (idx >= 0) {
      lines.push(text.slice(0, idx));
      return lines.join('\n');
    }
    lines.push(text);
  }
  return lines.join('\n');
}

export function unquoteLuauString(raw: string): string | undefined {
  const t = raw.trim().replace(/,$/, '').trim();
  if (
    (t.startsWith('"') && t.endsWith('"')) ||
    (t.startsWith("'") && t.endsWith("'"))
  ) {
    return t.slice(1, -1);
  }
  return undefined;
}

/** Module-level key completion when typing at shallow indent in return table. */
export function moduleKeyCompletionContext(
  before: string,
  lineText: string
): { typed: string; start: number } | undefined {
  // Only when line looks like a top-level field (modest indent)
  const indent = lineText.match(/^(\s*)/)?.[1].length ?? 0;
  if (indent > 4) {
    return undefined;
  }
  const match = /^(\s*)([A-Za-z_]\w*)$/.exec(before);
  if (!match) {
    return undefined;
  }
  return {
    typed: match[2] ?? '',
    start: match[1].length,
  };
}

/** Arg-definition table key completion inside `args = { { … } }`. */
export function argDefKeyCompletionContext(
  document: TextDocument,
  line: number,
  before: string
): { typed: string; start: number } | undefined {
  const source = document.getText();
  const argsIdx = source.search(/\bargs\s*=\s*\{/);
  if (argsIdx < 0) {
    return undefined;
  }
  // Rough: cursor line must fall within args block before apply
  let offset = 0;
  for (let i = 0; i < line; i++) {
    offset += document.lineAt(i).text.length + 1;
  }
  const applyIdx = source.search(/\bapply\s*=/);
  if (offset < argsIdx || (applyIdx >= 0 && offset > applyIdx)) {
    return undefined;
  }
  const match = /^(\s*)([A-Za-z_]\w*)$/.exec(before);
  if (!match) {
    return undefined;
  }
  return { typed: match[2] ?? '', start: match[1].length };
}
