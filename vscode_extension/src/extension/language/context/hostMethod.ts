import { hostMethodByName } from '../hostApiSurface';
import type { MethodSpecContext, TextDocument } from './types';

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
          const prefix = text.slice(0, j);
          const m = /\bc:([A-Za-z_]\w*)\s*\(\s*$/.exec(prefix);
          if (m) {
            return { method: m[1], openLine: i, openCol: j };
          }
          if (prefix.trim() === '' || /^\s*$/.test(prefix)) {
            for (let k = i; k >= Math.max(0, i - 3); k--) {
              const prev = k === i ? prefix : document.lineAt(k).text;
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
      // Closed table above
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
