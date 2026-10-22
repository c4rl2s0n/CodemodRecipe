import type * as vscode from 'vscode';
import {
  findSiblingQueryAssign,
  readLuauLongString,
  unquoteLuauString,
} from '../language/luauContext';

/** Find `path =` under a nearby `c:insert` / `c:replace` / `c:remove` call. */
export function findEditPathNearLine(
  document: vscode.TextDocument,
  line: number
): string | undefined {
  for (let i = line; i >= Math.max(0, line - 80); i--) {
    const text = document.lineAt(i).text;
    if (!/\bc:(insert|replace|remove)\s*\(/.test(text) && i !== line) {
      continue;
    }
    // Scan forward from this call for path =
    for (let j = i; j < Math.min(document.lineCount, i + 40); j++) {
      const t = document.lineAt(j).text;
      const pm = t.match(/\bpath\s*=\s*(.+?)(?:,\s*)?$/);
      if (pm) {
        const raw = pm[1].trim().replace(/,$/, '').trim();
        // Prefer string literals; skip args.file etc.
        const quoted = unquoteLuauString(raw);
        if (quoted) {
          return quoted;
        }
      }
      if (j > i && /\bc:[A-Za-z_]\w*\s*\(/.test(t)) {
        break;
      }
    }
    if (/\bc:(insert|replace|remove)\s*\(/.test(text)) {
      break;
    }
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
    if (/\bquery\s*=/.test(text)) {
      queryLine = i;
      break;
    }
  }
  if (queryLine < 0) {
    const sibling = findSiblingQueryAssign(document, line);
    if (sibling) {
      queryLine = sibling.line;
    }
  }
  if (queryLine < 0) {
    return undefined;
  }

  const header = document.lineAt(queryLine).text;
  const restMatch = header.match(/\bquery\s*=\s*(.*)$/);
  const rest = restMatch?.[1]?.trim() ?? '';
  let query: string | undefined;

  const long = readLuauLongString(document, queryLine, rest);
  if (long !== undefined) {
    query = long;
  } else {
    query = unquoteLuauString(rest);
  }

  if (!query) {
    return undefined;
  }
  return {
    query,
    ...scanCaptureAnchor(document, queryLine),
  };
}

function scanCaptureAnchor(
  document: vscode.TextDocument,
  fromLine: number
): { capture?: string; anchor?: 'start' | 'end' } {
  let capture: string | undefined;
  let anchor: 'start' | 'end' | undefined;
  for (let j = fromLine; j < Math.min(document.lineCount, fromLine + 60); j++) {
    const t = document.lineAt(j).text;
    if (j > fromLine && /\bc:[A-Za-z_]\w*\s*\(/.test(t)) {
      break;
    }
    const cm = t.match(/\bcapture\s*=\s*(.+?)(?:,\s*)?$/);
    if (cm) {
      capture = unquoteLuauString(cm[1].trim()) ?? cm[1].trim();
    }
    const am = t.match(/\banchor\s*=\s*['"]?(start|end)['"]?/);
    if (am) {
      anchor = am[1] as 'start' | 'end';
    }
  }
  return { capture, anchor };
}
