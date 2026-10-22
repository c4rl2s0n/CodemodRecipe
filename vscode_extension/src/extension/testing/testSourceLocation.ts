/**
 * Best-effort location of a test function inside a `*.test.luau` suite source,
 * for gutter run buttons and go-to-test. Convention-based: matches the
 * `<name> = function` key inside the `tests` table. Returns a 0-based line;
 * falls back to 0 (top of file) when the convention is not followed.
 *
 * Failure locations never use this — the host report carries the exact line.
 */
export function findTestFunctionLine(source: string, testName: string): number {
  const escaped = testName.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  // `[ \t]*`, not `\s*`: `\s` would consume the preceding newline and shift
  // `match.index` onto the previous line, off-by-one-ing the line count.
  const match = new RegExp(`^[ \\t]*(?:\\[")?${escaped}(?:"\\])?[ \\t]*=[ \\t]*function`, 'm').exec(source);
  if (!match || match.index === undefined) {
    return 0;
  }
  let line = 0;
  for (let i = 0; i < match.index; i++) {
    if (source.charCodeAt(i) === 10) {
      line++;
    }
  }
  return line;
}
