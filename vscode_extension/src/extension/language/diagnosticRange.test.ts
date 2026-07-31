import { describe, expect, it } from 'vitest';
import { diagnosticRangeParts } from './diagnosticRange';

describe('diagnosticRangeParts', () => {
  it('uses a whole-line file-level range when line and column are missing', () => {
    expect(diagnosticRangeParts(undefined, undefined)).toEqual({
      startLine: 0,
      startCol: 0,
      endLine: 0,
      endCol: Number.MAX_SAFE_INTEGER,
    });
  });

  it('converts 1-based host spans to a rest-of-line range', () => {
    expect(diagnosticRangeParts(4, 7)).toEqual({
      startLine: 3,
      startCol: 6,
      endLine: 3,
      endCol: Number.MAX_SAFE_INTEGER,
    });
  });

  it('defaults missing column to column 1 when line is set', () => {
    expect(diagnosticRangeParts(2, undefined)).toEqual({
      startLine: 1,
      startCol: 0,
      endLine: 1,
      endCol: Number.MAX_SAFE_INTEGER,
    });
  });
});
