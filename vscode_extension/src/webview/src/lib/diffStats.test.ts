import { describe, expect, it } from 'vitest';
import type { FilePreview } from '../shared';
import {
  countLines,
  lineChangeStats,
  statsForFile,
  statsForPatch,
  sumStats,
} from './diffStats';

describe('countLines', () => {
  it('treats empty as zero lines', () => {
    expect(countLines('')).toBe(0);
  });

  it('counts newline-separated lines', () => {
    expect(countLines('a\nb\nc')).toBe(3);
    expect(countLines('a\nb\n')).toBe(2);
  });
});

describe('lineChangeStats', () => {
  it('counts only changed lines (git-like)', () => {
    expect(lineChangeStats('old\nline', 'new\nline\ntwo')).toEqual({
      additions: 2,
      deletions: 1,
    });
  });

  it('uses +1/-1 for same-line replacements', () => {
    expect(lineChangeStats('foo', 'bar')).toEqual({
      additions: 1,
      deletions: 1,
    });
  });

  it('counts pure insertion as +1/-0 for shared prefix', () => {
    const oldText = [
      '/// Data models barrel.',
      'library;',
      "export 'host_entry.dart';",
    ].join('\n');
    const newText = [
      '/// Data models barrel.',
      'library;',
      "export 'host_entry.dart';",
      "export 'test_name.dart';",
    ].join('\n');
    expect(lineChangeStats(oldText, newText)).toEqual({
      additions: 1,
      deletions: 0,
    });
  });
});

describe('statsForPatch', () => {
  const file: FilePreview = {
    path: 'a.dart',
    kind: 'edit',
    isNew: false,
    skipped: false,
    original: 'hello world',
    modified: 'hello brave world',
    patches: [
      {
        index: 0,
        offset: 6,
        length: 0,
        replacement: 'brave ',
        description: 'insert',
      },
    ],
  };

  it('derives hunk stats from offset and replacement', () => {
    expect(statsForPatch(file, file.patches[0])).toEqual({
      additions: 1,
      deletions: 0,
    });
  });

  it('uses whole file for synthetic patch index -1', () => {
    expect(
      statsForPatch(file, {
        index: -1,
        offset: 0,
        length: 0,
        description: null,
      })
    ).toEqual(statsForFile(file));
  });
});

describe('sumStats', () => {
  it('aggregates patch hunks', () => {
    expect(
      sumStats([
        { additions: 2, deletions: 1 },
        { additions: 1, deletions: 0 },
      ])
    ).toEqual({ additions: 3, deletions: 1 });
  });
});
