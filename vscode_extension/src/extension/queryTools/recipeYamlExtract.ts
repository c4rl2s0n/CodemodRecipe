import type * as vscode from 'vscode';

/** Find `path:` under a nearby `edit:` block (walk upward then scan siblings). */
export function findEditPathNearLine(
  document: vscode.TextDocument,
  line: number
): string | undefined {
  for (let i = line; i >= 0; i--) {
    const text = document.lineAt(i).text;
    const m = text.match(/^(\s*)edit:\s*$/);
    if (!m) {
      continue;
    }
    const editIndent = m[1].length;
    for (let j = i + 1; j < document.lineCount; j++) {
      const t = document.lineAt(j).text;
      const ind = t.match(/^(\s*)/)?.[1].length ?? 0;
      if (t.trim() && ind <= editIndent) {
        break;
      }
      const pm = t.match(/^\s*path:\s*(.+)\s*$/);
      if (pm) {
        return unquote(pm[1].trim());
      }
    }
    break;
  }
  return undefined;
}

export function extractQueryCaptureAnchor(
  document: vscode.TextDocument,
  line: number
): { query: string; capture?: string; anchor?: 'start' | 'end' } | undefined {
  let queryLine = -1;
  for (let i = line; i >= Math.max(0, line - 120); i--) {
    const text = document.lineAt(i).text;
    if (/^\s*query:\s*/.test(text)) {
      queryLine = i;
      break;
    }
  }
  if (queryLine < 0) {
    return undefined;
  }

  const header = document.lineAt(queryLine).text;
  const baseIndent = header.match(/^(\s*)/)?.[1].length ?? 0;
  let query: string | undefined;

  const inline = header.match(/^\s*query:\s*(.+)\s*$/);
  if (inline && !header.includes('|')) {
    query = unquote(inline[1].trim());
  } else {
    const rawLines: string[] = [];
    for (let j = queryLine + 1; j < document.lineCount; j++) {
      const t = document.lineAt(j).text;
      const ind = t.match(/^(\s*)/)?.[1].length ?? 0;
      if (t.trim() && ind <= baseIndent) {
        break;
      }
      rawLines.push(t);
    }
    query = stripCommonIndent(rawLines).trimEnd();
  }

  if (!query) {
    return undefined;
  }
  return {
    query,
    ...scanCaptureAnchor(document, queryLine, baseIndent),
  };
}

function scanCaptureAnchor(
  document: vscode.TextDocument,
  fromLine: number,
  baseIndent: number
): { capture?: string; anchor?: 'start' | 'end' } {
  let capture: string | undefined;
  let anchor: 'start' | 'end' | undefined;
  for (let j = fromLine; j < Math.min(document.lineCount, fromLine + 60); j++) {
    const t = document.lineAt(j).text;
    const ind = t.match(/^(\s*)/)?.[1].length ?? 0;
    if (j > fromLine && t.trim() && ind < baseIndent) {
      break;
    }
    if (/^\s*-\s+(insert|replace|remove):/.test(t) && j > fromLine + 1) {
      // next op
      if (ind <= baseIndent) {
        break;
      }
    }
    const cm = t.match(/^\s*capture:\s*(.+)\s*$/);
    if (cm) {
      capture = unquote(cm[1].trim());
    }
    const am = t.match(/^\s*anchor:\s*(start|end)\s*$/);
    if (am) {
      anchor = am[1] as 'start' | 'end';
    }
  }
  return { capture, anchor };
}

function unquote(s: string): string {
  if (
    (s.startsWith('"') && s.endsWith('"')) ||
    (s.startsWith("'") && s.endsWith("'"))
  ) {
    return s.slice(1, -1);
  }
  return s;
}

function stripCommonIndent(lines: string[]): string {
  const nonEmpty = lines.filter((l) => l.trim().length);
  if (!nonEmpty.length) {
    return '';
  }
  let min = Infinity;
  for (const l of nonEmpty) {
    const m = l.match(/^(\s*)/);
    min = Math.min(min, m?.[1].length ?? 0);
  }
  return lines.map((l) => (l.length >= min ? l.slice(min) : l)).join('\n');
}
