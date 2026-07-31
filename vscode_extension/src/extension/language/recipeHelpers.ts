import type * as vscode from 'vscode';

type TextDocument = Pick<vscode.TextDocument, 'lineAt' | 'lineCount'>;

export function documentTopLevelId(document: TextDocument): string | undefined {
  for (let i = 0; i < Math.min(document.lineCount, 40); i++) {
    const text = document.lineAt(i).text;
    // Only unindented top-level id (not nested recipe step ids).
    const match = text.match(/^id:\s*['"]?([A-Za-z_][\w./-]*)['"]?\s*$/);
    if (match) {
      return match[1];
    }
  }
  return undefined;
}

export function matchRecipeReference(
  line: string,
  character: number
): string | undefined {
  const patterns = [
    /\brecipe:\s*['"]?([A-Za-z_][\w./-]*)['"]?/,
    /^\s*id:\s*['"]?([A-Za-z_][\w./-]*)['"]?\s*$/,
  ];
  for (const pattern of patterns) {
    const match = pattern.exec(line);
    if (!match || match.index === undefined) {
      continue;
    }
    const start = match.index + match[0].indexOf(match[1]);
    const end = start + match[1].length;
    if (character >= start && character <= end) {
      return match[1];
    }
  }
  return undefined;
}

export function matchTemplateFile(
  line: string,
  character: number
): string | undefined {
  const match = /templateFile:\s*['"]?([^\s'"]+)['"]?/.exec(line);
  if (!match || match.index === undefined) {
    return undefined;
  }
  const start = match.index + match[0].indexOf(match[1]);
  const end = start + match[1].length;
  if (character >= start && character <= end) {
    return match[1];
  }
  return undefined;
}

export function isUnderRecipeMapping(
  document: TextDocument,
  line: number
): boolean {
  for (let i = line - 1; i >= Math.max(0, line - 12); i--) {
    const text = document.lineAt(i).text;
    if (/^\s*recipe:\s*$/.test(text) || /^\s*-\s*recipe:\s*$/.test(text)) {
      return true;
    }
    if (/^\s*-\s*(edit|create|delete):/.test(text)) {
      return false;
    }
  }
  return false;
}

export function findNearbyRecipeStepId(
  document: TextDocument,
  line: number
): string | undefined {
  for (let i = line; i >= Math.max(0, line - 20); i--) {
    const text = document.lineAt(i).text;
    const inline = text.match(
      /^\s*(?:-\s*)?recipe:\s*['"]?([A-Za-z_][\w./-]*)['"]?\s*$/
    );
    if (inline) {
      return inline[1];
    }
    const idLine = text.match(/^\s*id:\s*['"]?([A-Za-z_][\w./-]*)['"]?\s*$/);
    if (idLine && isUnderRecipeMapping(document, i)) {
      return idLine[1];
    }
  }
  return undefined;
}

export function inWithBlock(document: TextDocument, line: number): boolean {
  for (let i = line; i >= Math.max(0, line - 30); i--) {
    const text = document.lineAt(i).text;
    if (/^\s*with:\s*$/.test(text)) {
      return true;
    }
    if (/^\s*(?:-\s*)?(edit|create|delete|recipe):/.test(text) && i < line) {
      return false;
    }
  }
  return false;
}

/**
 * Collect keys already present under the enclosing `with:` mapping.
 */
export function collectSetWithKeys(
  document: TextDocument,
  line: number
): Set<string> {
  const keys = new Set<string>();
  let withIndent = -1;
  let withLine = -1;
  for (let i = line; i >= Math.max(0, line - 40); i--) {
    const text = document.lineAt(i).text;
    const match = text.match(/^(\s*)with:\s*$/);
    if (match) {
      withIndent = match[1].length;
      withLine = i;
      break;
    }
  }
  if (withLine < 0) {
    return keys;
  }

  let entryIndent: number | undefined;
  for (let i = withLine + 1; i < document.lineCount; i++) {
    const text = document.lineAt(i).text;
    if (text.trim() === '') {
      continue;
    }
    const indent = text.match(/^(\s*)/)?.[1].length ?? 0;
    if (indent <= withIndent) {
      break;
    }
    if (entryIndent === undefined) {
      entryIndent = indent;
    }
    if (indent !== entryIndent) {
      continue;
    }
    const keyMatch = text.match(/^\s*([A-Za-z_]\w*)\s*:/);
    if (keyMatch) {
      keys.add(keyMatch[1]);
    }
  }
  return keys;
}

export function parseArgNames(source: string): string[] {
  const names: string[] = [];
  const argsBlock = source.match(
    /\nargs:\s*\n((?:[ \t]+-[\s\S]*?)?)(?=\n\w|\n*$)/
  );
  const block = argsBlock?.[1] ?? source;
  const namePattern =
    /(?:^|\n)[ \t]*-[ \t]*(?:\{[ \t]*)?name:\s*['"]?([A-Za-z_]\w*)['"]?/g;
  let match: RegExpExecArray | null;
  while ((match = namePattern.exec(block)) !== null) {
    names.push(match[1]);
  }
  const alt = /(?:^|\n)[ \t]+name:\s*['"]?([A-Za-z_]\w*)['"]?/g;
  while ((match = alt.exec(source)) !== null) {
    if (!names.includes(match[1])) {
      names.push(match[1]);
    }
  }
  return names;
}

/** First top-level `query:` line index, if any. */
export function firstTopLevelQueryLine(
  document: TextDocument
): number | undefined {
  for (let i = 0; i < document.lineCount; i++) {
    const text = document.lineAt(i).text;
    if (/^query:\s*/.test(text) || /^\s{2}query:\s*/.test(text)) {
      // Prefer recipe-level queries map or first edit query — use first indented query under steps.
      if (/^\s+query:\s*/.test(text)) {
        return i;
      }
    }
  }
  for (let i = 0; i < document.lineCount; i++) {
    if (/^\s+query:\s*/.test(document.lineAt(i).text)) {
      return i;
    }
  }
  return undefined;
}
