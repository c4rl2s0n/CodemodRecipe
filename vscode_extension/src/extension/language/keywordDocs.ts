import * as fs from 'fs';
import * as vscode from 'vscode';

export interface KeywordDocEntry {
  kind: string;
  wire: string;
  parent?: string;
  schemaPath?: string;
  description: string;
}

let cached: KeywordDocEntry[] | undefined;

export function loadKeywordDocs(extensionUri: vscode.Uri): KeywordDocEntry[] {
  if (cached) {
    return cached;
  }
  const file = vscode.Uri.joinPath(
    extensionUri,
    'schemas',
    'generated-keyword-docs.json'
  );
  try {
    const raw = fs.readFileSync(file.fsPath, 'utf8');
    cached = JSON.parse(raw) as KeywordDocEntry[];
  } catch {
    cached = [];
  }
  return cached;
}

export function lookupKeywordHover(
  docs: KeywordDocEntry[],
  line: string,
  character: number
): string | undefined {
  const keyMatch = /(?<![\w-])([A-Za-z_][\w]*)(?![\w-])\s*:/g;
  for (const match of line.matchAll(keyMatch)) {
    const wire = match[1];
    const start = match.index! + match[0].indexOf(wire);
    const end = start + wire.length;
    if (character >= start && character <= end) {
      const doc = docs.find((d) => !d.parent && d.wire === wire);
      return doc?.description;
    }
  }

  const kvMatch =
    /(?<![\w-])([A-Za-z_][\w]*)\s*:\s*['"]?([A-Za-z_][\w]*)['"]?/g;
  for (const match of line.matchAll(kvMatch)) {
    const parent = match[1];
    const value = match[2];
    const valueStart = match.index! + match[0].indexOf(value);
    const valueEnd = valueStart + value.length;
    if (character >= valueStart && character <= valueEnd) {
      const doc = docs.find((d) => d.parent === parent && d.wire === value);
      return doc?.description;
    }
  }

  return undefined;
}
