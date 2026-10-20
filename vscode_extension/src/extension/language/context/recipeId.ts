import type { RecipeIdContext, TextDocument } from './types';

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

export function documentTopLevelId(document: TextDocument): string | undefined {
  const text = document.getText();
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
