import type { TextDocument } from './types';

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
