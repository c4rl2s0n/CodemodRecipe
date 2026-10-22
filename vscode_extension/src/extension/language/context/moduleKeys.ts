/** Module-level key completion when typing at shallow indent in return table. */
export function moduleKeyCompletionContext(
  before: string,
  lineText: string
): { typed: string; start: number } | undefined {
  const indent = lineText.match(/^(\s*)/)?.[1].length ?? 0;
  if (indent > 4) {
    return undefined;
  }
  const match = /^(\s*)([A-Za-z_]\w*)$/.exec(before);
  if (!match) {
    return undefined;
  }
  return {
    typed: match[2] ?? '',
    start: match[1].length,
  };
}
