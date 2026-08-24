import { describe, expect, it } from 'vitest';
import type { FilePreview } from '../shared';
import { buildSelection, defaultFileSelections } from './selection';

const file: FilePreview = {
  path: 'a.dart',
  kind: 'edit',
  isNew: false,
  skipped: false,
  original: 'old',
  modified: 'new',
};

describe('defaultFileSelections', () => {
  it('includes all files by default', () => {
    const selections = defaultFileSelections([file]);
    expect(selections).toHaveLength(1);
    expect(selections[0]).toEqual({ path: 'a.dart', include: true });
  });
});

describe('buildSelection', () => {
  it('includes checked files only', () => {
    const selections = [
      { path: 'a.dart', include: true },
      { path: 'b.dart', include: false },
    ];
    const payload = buildSelection(selections);
    expect(payload.files).toEqual({
      'a.dart': { include: true },
      'b.dart': { include: false },
    });
  });
});
