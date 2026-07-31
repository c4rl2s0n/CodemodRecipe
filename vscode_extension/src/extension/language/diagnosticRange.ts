export type DiagnosticRangeParts = {
  startLine: number;
  startCol: number;
  endLine: number;
  endCol: number;
};

/** Pure range helper (0-based). */
export function diagnosticRangeParts(
  line: number | undefined,
  column: number | undefined
): DiagnosticRangeParts {
  if (line === undefined && column === undefined) {
    // File-level: whole first line rather than a misleading 1-char squiggle at (1,1).
    return {
      startLine: 0,
      startCol: 0,
      endLine: 0,
      endCol: Number.MAX_SAFE_INTEGER,
    };
  }
  const startLine = Math.max(0, (line ?? 1) - 1);
  const startCol = Math.max(0, (column ?? 1) - 1);
  // Highlight from the reported column through rest-of-line (editor clamps).
  return {
    startLine,
    startCol,
    endLine: startLine,
    endCol: Number.MAX_SAFE_INTEGER,
  };
}
