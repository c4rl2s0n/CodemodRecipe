import { describe, expect, it } from 'vitest';
import { buildSelection, defaultFileSelections } from './selection';
import type { FilePreview } from '../types';

const file: FilePreview = {
  path: 'lib/a.dart',
  kind: 'edit',
  isNew: false,
  skipped: false,
  patches: [
    {
      index: 0,
      offset: 1,
      length: 2,
      description: 'patch',
      replacement: 'x',
    },
  ],
};

describe('buildSelection', () => {
  it('includes only checked patches', () => {
    const selections = defaultFileSelections([file]);
    selections[0].patches[0].include = false;
    const payload = buildSelection(selections);
    expect(payload.files['lib/a.dart']).toEqual({
      include: true,
      patches: [],
    });
  });
});
