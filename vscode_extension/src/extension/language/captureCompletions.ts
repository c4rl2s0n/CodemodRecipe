/**
 * Pure helpers: suggest `capture:` values from `@names` in the sibling `query`.
 * No tree-sitter — regex scrape of inline query text or a `.scm` file path.
 */

export type LineSource = {
  lineCount: number;
  lineText(line: number): string;
};

function lineIndent(text: string): number {
  const match = text.match(/^(\s*)/);
  return match?.[1].length ?? 0;
}

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

/**
 * True when the scalar looks like a path to a `.scm` file (not an S-expression
 * or query-library id).
 */
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

/**
 * Query-library refs (`libId.key`) and opaque chains — skip file / scrape.
 */
export function isLikelyQueryLibraryOrChain(value: string): boolean {
  const v = value.trim().replace(/^['"]|['"]$/g, '');
  if (!v || v.startsWith('(') || v.includes('@') || /\.scm$/i.test(v)) {
    return false;
  }
  // Single dotted id without path separators (e.g. dart.bodies.method).
  if (v.includes('/') || v.includes('\\')) {
    return false;
  }
  return /^[A-Za-z_][\w]*(?:\.[A-Za-z_][\w]*)+$/.test(v);
}

function unquoteScalar(raw: string): string {
  const t = raw.trim();
  if (
    (t.startsWith('"') && t.endsWith('"')) ||
    (t.startsWith("'") && t.endsWith("'"))
  ) {
    return t.slice(1, -1);
  }
  return t;
}

/**
 * Locate a sibling `query:` field at the same indent as `captureLine`
 * (still under the same parent mapping).
 */
export function findSiblingQueryField(
  document: LineSource,
  captureLine: number
): { line: number; indent: number; rest: string } | undefined {
  if (captureLine < 0 || captureLine >= document.lineCount) {
    return undefined;
  }
  const indent = lineIndent(document.lineText(captureLine));

  const tryLine = (i: number): { line: number; indent: number; rest: string } | undefined => {
    if (i < 0 || i >= document.lineCount || i === captureLine) {
      return undefined;
    }
    const lt = document.lineText(i);
    if (lt.trim() === '' || lt.trim().startsWith('#')) {
      return undefined;
    }
    const ind = lineIndent(lt);
    if (ind < indent) {
      return undefined; // signal stop via sentinel — handled by callers
    }
    if (ind !== indent) {
      return undefined;
    }
    const m = lt.match(/^\s*query:\s*(.*)$/);
    if (!m) {
      return undefined;
    }
    return { line: i, indent: ind, rest: m[1] ?? '' };
  };

  // Prefer earlier siblings (query usually above capture).
  for (let i = captureLine - 1; i >= 0; i--) {
    const lt = document.lineText(i);
    if (lt.trim() === '') {
      continue;
    }
    const ind = lineIndent(lt);
    if (ind < indent) {
      break;
    }
    const hit = tryLine(i);
    if (hit) {
      return hit;
    }
  }
  for (let i = captureLine + 1; i < document.lineCount; i++) {
    const lt = document.lineText(i);
    if (lt.trim() === '') {
      continue;
    }
    const ind = lineIndent(lt);
    if (ind < indent) {
      break;
    }
    const hit = tryLine(i);
    if (hit) {
      return hit;
    }
  }
  return undefined;
}

/**
 * Read a YAML `|` / `>` block body following `queryLine` (lines more indented
 * than the `query:` key).
 */
export function readBlockScalarBody(
  document: LineSource,
  queryLine: number,
  queryIndent: number
): string {
  const lines: string[] = [];
  for (let i = queryLine + 1; i < document.lineCount; i++) {
    const lt = document.lineText(i);
    if (lt.trim() === '') {
      // Blank lines inside the block: keep if next content stays indented.
      let nextInd = -1;
      for (let j = i + 1; j < document.lineCount; j++) {
        const n = document.lineText(j);
        if (n.trim() === '') {
          continue;
        }
        nextInd = lineIndent(n);
        break;
      }
      if (nextInd > queryIndent) {
        lines.push('');
        continue;
      }
      break;
    }
    if (lineIndent(lt) <= queryIndent) {
      break;
    }
    lines.push(lt);
  }
  return lines.join('\n');
}

export type ResolvedSiblingQuery =
  | { kind: 'inline'; text: string }
  | { kind: 'scmPath'; path: string }
  | { kind: 'unsupported' };

/**
 * Resolve the sibling `query` of a `capture:` line into inline text, an `.scm`
 * path, or unsupported (library / chain / missing).
 */
export function resolveSiblingQuery(
  document: LineSource,
  captureLine: number
): ResolvedSiblingQuery | undefined {
  const field = findSiblingQueryField(document, captureLine);
  if (!field) {
    return undefined;
  }
  const rest = field.rest.trim();
  if (rest === '|' || rest === '>' || rest.startsWith('|') || rest.startsWith('>')) {
    const body = readBlockScalarBody(document, field.line, field.indent);
    return { kind: 'inline', text: body };
  }
  if (!rest) {
    // Rare: `query:` with indented body and no `|` — treat indented follow as body.
    const body = readBlockScalarBody(document, field.line, field.indent);
    if (body.trim()) {
      return { kind: 'inline', text: body };
    }
    return undefined;
  }
  const scalar = unquoteScalar(rest);
  if (isLikelyScmPath(scalar)) {
    return { kind: 'scmPath', path: scalar };
  }
  if (isLikelyQueryLibraryOrChain(scalar)) {
    return { kind: 'unsupported' };
  }
  return { kind: 'inline', text: scalar };
}

/**
 * Capture names offered for `capture:` at `captureLine`, filtered by prefix.
 * `readScmFile` is optional; when omitted, `.scm` paths yield no names.
 */
export function collectCaptureNameCompletions(
  document: LineSource,
  captureLine: number,
  typedPrefix: string,
  readScmFile?: (relativePath: string) => string | undefined
): string[] {
  const resolved = resolveSiblingQuery(document, captureLine);
  if (!resolved || resolved.kind === 'unsupported') {
    return [];
  }
  let source = '';
  if (resolved.kind === 'inline') {
    source = resolved.text;
  } else if (readScmFile) {
    source = readScmFile(resolved.path) ?? '';
  }
  if (!source) {
    return [];
  }
  const prefix = typedPrefix.trim();
  return extractCaptureNamesFromQuerySource(source).filter(
    (name) => !prefix || name.startsWith(prefix)
  );
}
