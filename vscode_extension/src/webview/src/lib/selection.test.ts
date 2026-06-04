import { describe, expect, it } from 'vitest';
import {
  buildSelection,
  defaultFileSelections,
  setFileInclude,
  setPatchInclude,
} from './selection';
import type { FilePreview } from '../shared';

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
    selections[0].include = false;
    const payload = buildSelection(selections);
    expect(payload.files['lib/a.dart']).toEqual({
      include: false,
      patches: [],
    });
  });
});

describe('file/patch include sync', () => {
  it('file toggle sets all patch includes', () => {
    const base = defaultFileSelections([file])[0];
    const off = setFileInclude(base, false);
    expect(off.include).toBe(false);
    expect(off.patches.every((p) => !p.include)).toBe(true);

    const on = setFileInclude(off, true);
    expect(on.include).toBe(true);
    expect(on.patches.every((p) => p.include)).toBe(true);
  });

  it('last patch excluded excludes file; any patch included includes file', () => {
    const base = defaultFileSelections([file])[0];
    const excluded = setPatchInclude(base, 0, false);
    expect(excluded.include).toBe(false);
    expect(excluded.patches[0].include).toBe(false);

    const included = setPatchInclude(excluded, 0, true);
    expect(included.include).toBe(true);
    expect(included.patches[0].include).toBe(true);
  });
});
