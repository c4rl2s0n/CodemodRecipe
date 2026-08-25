/**
 * Pure helpers: suggest `capture` values from `@names` in a sibling `query`.
 * Supports Luau `query = [[...]]` / `query = "..."` and legacy `.scm` paths.
 */

export type LineSource = {
  lineCount: number;
  lineText(line: number): string;
};

/** Unique capture names from `@foo` tokens in a query source string. */
export function extractCaptureNamesFromQuerySource(querySource: string): string[] {
  const names: string[] = [];
  const seen = new Set<string>();
  const re = /@([A-Za-z_][\w]*)/g;
  let match: RegExpExecArray | null;
  while ((match = re.exec(querySource)) !== null) {
    const name = match[1];
    if (!seen.has(name)) {
      seen.add(name);
      names.push(name);
    }
  }
  return names;
}

export function isLikelyScmPath(value: string): boolean {
  const v = value.trim().replace(/^['"]|['"]$/g, '');
  if (!v || v.includes('\n')) {
    return false;
  }
  if (v.startsWith('(') || v.includes('@')) {
    return false;
  }
  return /\.scm$/i.test(v);
}

export function isLikelyQueryLibraryOrChain(value: string): boolean {
  const v = value.trim().replace(/^['"]|['"]$/g, '');
  if (!v || v.startsWith('(') || v.includes('@') || /\.scm$/i.test(v)) {
    return false;
  }
  if (v.includes('/') || v.includes('\\')) {
    return false;
  }
  return /^[A-Za-z_][\w]*(?:\.[A-Za-z_][\w]*)+$/.test(v);
}

function unquoteScalar(raw: string): string {
  const t = raw.trim().replace(/,$/, '').trim();
  if (
    (t.startsWith('"') && t.endsWith('"')) ||
    (t.startsWith("'") && t.endsWith("'"))
  ) {
    return t.slice(1, -1);
  }
  return t;
}

export type ResolvedSiblingQuery =
  | { kind: 'inline'; text: string }
  | { kind: 'scmPath'; path: string }
  | { kind: 'queryId'; id: string }
  | { kind: 'unsupported' };

function readLuauLongString(
  document: LineSource,
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
    const text = document.lineText(i);
    const idx = text.indexOf(close);
    if (idx >= 0) {
      lines.push(text.slice(0, idx));
      return lines.join('\n');
    }
    lines.push(text);
  }
  return lines.join('\n');
}

/**
 * Locate sibling `query =` near a `capture =` line (Luau table fields).
 */
export function findSiblingQueryAssign(
  document: LineSource,
  captureLine: number
): { line: number; rest: string } | undefined {
  if (captureLine < 0 || captureLine >= document.lineCount) {
    return undefined;
  }
  for (let i = captureLine - 1; i >= Math.max(0, captureLine - 60); i--) {
    const lt = document.lineText(i);
    if (lt.trim() === '') {
      continue;
    }
    const m = lt.match(/^\s*query\s*=\s*(.*)$/);
    if (m) {
      return { line: i, rest: m[1] ?? '' };
    }
    if (/\bc:[A-Za-z_]\w*\s*\(/.test(lt)) {
      break;
    }
  }
  return undefined;
}

export function resolveSiblingQuery(
  document: LineSource,
  captureLine: number
): ResolvedSiblingQuery | undefined {
  const field = findSiblingQueryAssign(document, captureLine);
  if (!field) {
    return undefined;
  }
  const rest = field.rest.trim();
  const long = readLuauLongString(document, field.line, rest);
  if (long !== undefined) {
    return { kind: 'inline', text: long };
  }
  if (!rest) {
    return undefined;
  }
  const scalar = unquoteScalar(rest);
  if (isLikelyScmPath(scalar)) {
    return { kind: 'scmPath', path: scalar };
  }
  if (isLikelyQueryLibraryOrChain(scalar)) {
    return { kind: 'queryId', id: scalar };
  }
  if (scalar.startsWith('(') || scalar.includes('@')) {
    return { kind: 'inline', text: scalar };
  }
  return { kind: 'queryId', id: scalar };
}

export function collectCaptureNameCompletions(
  document: LineSource,
  captureLine: number,
  typedPrefix: string,
  readScmFile?: (relativePath: string) => string | undefined,
  resolveQueryId?: (id: string) => string | undefined
): string[] {
  const resolved = resolveSiblingQuery(document, captureLine);
  if (!resolved || resolved.kind === 'unsupported') {
    return [];
  }
  let source = '';
  if (resolved.kind === 'inline') {
    source = resolved.text;
  } else if (resolved.kind === 'scmPath' && readScmFile) {
    source = readScmFile(resolved.path) ?? '';
  } else if (resolved.kind === 'queryId' && resolveQueryId) {
    source = resolveQueryId(resolved.id) ?? '';
  }
  if (!source) {
    return [];
  }
  const prefix = typedPrefix.trim();
  return extractCaptureNamesFromQuerySource(source).filter(
    (name) => !prefix || name.startsWith(prefix)
  );
}
