export type RecipeIdCompletion = {
  label: string;
  insertText: string;
  fullPath: string;
  hasChildren: boolean;
};

export type RecipeIdCompletionContext = {
  typed: string;
  start: number;
};

function splitRecipeId(id: string): string[] {
  return id.split('.').filter(Boolean);
}

/**
 * Detect partial recipe / asset id after Luau `id =`, `c:run(`, `c:map(`,
 * `c:var(`, or `c:queryLib(`.
 */
export function recipeIdCompletionContext(
  before: string
): RecipeIdCompletionContext | undefined {
  const idChars = String.raw`[\w./-]`;
  const patterns = [
    new RegExp(String.raw`\bc:run\s*\(\s*['"](${idChars}*)$`),
    new RegExp(String.raw`\bc:map\s*\(\s*['"](${idChars}*)$`),
    new RegExp(String.raw`\bc:var\s*\(\s*['"](${idChars}*)$`),
    new RegExp(String.raw`\bc:queryLib\s*\(\s*['"](${idChars}*)$`),
    new RegExp(String.raw`(?:^|[\s,{])id\s*=\s*['"](${idChars}*)$`),
  ];
  for (const re of patterns) {
    const match = re.exec(before);
    if (!match || match.index === undefined) {
      continue;
    }
    const typed = match[1] ?? '';
    const start = match.index + match[0].length - typed.length;
    return { typed, start };
  }
  return undefined;
}

export function collectRecipeIdCompletions(
  recipeIds: readonly string[],
  typed: string
): RecipeIdCompletion[] {
  const normalized = typed.trim();
  const endsWithDot = normalized.endsWith('.');
  const parts = splitRecipeId(normalized);
  const parentSegments = endsWithDot ? parts : parts.slice(0, -1);
  const currentFragment = endsWithDot ? '' : (parts.at(-1) ?? '');

  const suggestions = new Map<string, RecipeIdCompletion>();

  for (const recipeId of recipeIds) {
    const segments = splitRecipeId(recipeId);
    if (segments.length <= parentSegments.length) {
      continue;
    }
    const matchesParent = parentSegments.every(
      (segment, index) => segments[index] === segment
    );
    if (!matchesParent) {
      continue;
    }
    const nextSegment = segments[parentSegments.length];
    if (!nextSegment.startsWith(currentFragment)) {
      continue;
    }
    if (suggestions.has(nextSegment)) {
      const existing = suggestions.get(nextSegment)!;
      existing.hasChildren ||= segments.length > parentSegments.length + 1;
      continue;
    }
    suggestions.set(nextSegment, {
      label: nextSegment,
      insertText: nextSegment,
      fullPath: [...parentSegments, nextSegment].join('.'),
      hasChildren: segments.length > parentSegments.length + 1,
    });
  }

  return [...suggestions.values()].sort((a, b) => a.label.localeCompare(b.label));
}
