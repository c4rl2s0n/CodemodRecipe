import type { TextDocument } from './types';

/**
 * Arg names from the recipe module `args = { { name = "…" }, … }` table.
 */
export function parseRecipeArgNames(source: string): string[] {
  const names: string[] = [];
  const seen = new Set<string>();
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

/** `args.` completion prefix (apply's second parameter). */
export function argsFieldCompletionContext(
  before: string
): { typed: string; start: number } | undefined {
  const match = /\bargs\.([A-Za-z_]\w*)$/.exec(before);
  if (!match || match.index === undefined) {
    return undefined;
  }
  const typed = match[1] ?? '';
  const start = match.index + match[0].length - typed.length;
  return { typed, start };
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
    if (
      m &&
      !['path', 'query', 'capture', 'text', 'language', 'anchor'].includes(m[1])
    ) {
      keys.add(m[1]);
    }
    const any = text.matchAll(/(?:^|[\s,{])([A-Za-z_]\w*)\s*=/g);
    for (const match of any) {
      keys.add(match[1]);
    }
  }
  return keys;
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
