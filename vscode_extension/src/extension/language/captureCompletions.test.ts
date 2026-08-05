import { describe, expect, it } from 'vitest';
import {
  collectCaptureNameCompletions,
  extractCaptureNamesFromQuerySource,
  findSiblingQueryField,
  isLikelyQueryLibraryOrChain,
  isLikelyScmPath,
  resolveSiblingQuery,
  type LineSource,
} from './captureCompletions';

function doc(lines: string[]): LineSource {
  return {
    lineCount: lines.length,
    lineText(line: number) {
      return lines[line] ?? '';
    },
  };
}

describe('extractCaptureNamesFromQuerySource', () => {
  it('collects unique @names in order', () => {
    const src = `
(class_definition
  name: (identifier) @className
  body: (class_body (block) @body)
  (#eq? @className "X"))
`;
    expect(extractCaptureNamesFromQuerySource(src)).toEqual([
      'className',
      'body',
    ]);
  });
});

describe('isLikelyScmPath / library', () => {
  it('detects .scm paths', () => {
    expect(isLikelyScmPath('queries/body.scm')).toBe(true);
    expect(isLikelyScmPath('"(block) @a"')).toBe(false);
  });

  it('detects query-library ids', () => {
    expect(isLikelyQueryLibraryOrChain('dart.bodies.method')).toBe(true);
    expect(isLikelyQueryLibraryOrChain('(block) @body')).toBe(false);
    expect(isLikelyQueryLibraryOrChain('foo/bar.scm')).toBe(false);
  });
});

describe('sibling query resolution', () => {
  const insertLines = [
    'steps:',
    '  - edit:',
    '      path: "{{file}}"',
    '      ops:',
    '        - insert:',
    '            query: |',
    '              (block) @body',
    '              (identifier) @name',
    '            capture: ',
    '            anchor: end',
    '            text: "x"',
  ];

  it('finds query above capture', () => {
    const field = findSiblingQueryField(doc(insertLines), 8);
    expect(field?.rest.trim()).toBe('|');
    expect(field?.line).toBe(5);
  });

  it('resolves block scalar captures', () => {
    const names = collectCaptureNameCompletions(doc(insertLines), 8, '');
    expect(names).toEqual(['body', 'name']);
  });

  it('filters by typed prefix', () => {
    const names = collectCaptureNameCompletions(doc(insertLines), 8, 'b');
    expect(names).toEqual(['body']);
  });

  it('resolves same-line inline query', () => {
    const lines = [
      '        - replace:',
      '            query: "(declaration) @member"',
      '            capture: mem',
    ];
    expect(collectCaptureNameCompletions(doc(lines), 2, 'mem')).toEqual([
      'member',
    ]);
  });

  it('returns scmPath for .scm sibling', () => {
    const lines = [
      '        - remove:',
      '            query: queries/foo.scm',
      '            capture: ',
    ];
    expect(resolveSiblingQuery(doc(lines), 2)).toEqual({
      kind: 'scmPath',
      path: 'queries/foo.scm',
    });
  });

  it('reads .scm via callback', () => {
    const lines = [
      '        - insert:',
      '            query: body.scm',
      '            capture: ',
      '            anchor: end',
    ];
    const names = collectCaptureNameCompletions(
      doc(lines),
      2,
      '',
      (p) => (p === 'body.scm' ? '(block) @body\n' : undefined)
    );
    expect(names).toEqual(['body']);
  });

  it('skips query-library refs', () => {
    const lines = [
      '        - insert:',
      '            query: dart.bodies.method',
      '            capture: ',
    ];
    expect(resolveSiblingQuery(doc(lines), 2)).toEqual({ kind: 'unsupported' });
    expect(collectCaptureNameCompletions(doc(lines), 2, '')).toEqual([]);
  });
});
