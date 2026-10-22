import { describe, expect, it } from 'vitest';
import {
  buildCoverageTree,
  fileTint,
  rollupTint,
} from './coverageTree';

describe('fileTint', () => {
  it('maps exact ratios', () => {
    expect(fileTint(1)).toBe('green');
    expect(fileTint(0)).toBe('red');
    expect(fileTint(0.5)).toBe('yellow');
  });
});

describe('rollupTint', () => {
  it('all green / all red / mixed', () => {
    expect(rollupTint(['green', 'green'])).toBe('green');
    expect(rollupTint(['red', 'red'])).toBe('red');
    expect(rollupTint(['green', 'red'])).toBe('yellow');
    expect(rollupTint(['yellow'])).toBe('yellow');
  });
});

describe('buildCoverageTree', () => {
  it('builds nested dirs and rolls up tints', () => {
    const tree = buildCoverageTree([
      {
        path: 'lib/a.dart',
        ratio: 1,
        coveredNonblank: 10,
        gapNonblank: 0,
        gaps: [],
      },
      {
        path: 'lib/nested/b.dart',
        ratio: 0,
        coveredNonblank: 0,
        gapNonblank: 5,
        gaps: [{ from: 0, to: 5 }],
      },
      {
        path: 'src/c.dart',
        ratio: 0.5,
        coveredNonblank: 1,
        gapNonblank: 1,
      },
    ]);
    expect(tree.map((n) => n.name).sort()).toEqual(['lib', 'src']);
    const lib = tree.find((n) => n.name === 'lib')!;
    expect(lib.kind).toBe('dir');
    expect(lib.tint).toBe('yellow'); // green + red children
    const nested = lib.children?.find((c) => c.name === 'nested');
    expect(nested?.tint).toBe('red');
    const a = lib.children?.find((c) => c.name === 'a.dart');
    expect(a?.tint).toBe('green');
    expect(a?.kind).toBe('file');
    const src = tree.find((n) => n.name === 'src')!;
    expect(src.tint).toBe('yellow');
  });
});
