import { describe, expect, it } from 'vitest';
import {
  collectCaptureNameCompletions,
  extractCaptureNamesFromQuerySource,
  findSiblingQueryAssign,
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

describe('sibling query resolution (Luau)', () => {
  const insertLines = [
    '		c:insert({',
    '			path = args.file,',
    '			query = [[',
    '(block) @body',
    '(identifier) @name',
    ']],',
    '			capture = "',
    '			anchor = "end",',
    '			text = "x",',
    '		})',
  ];

  it('finds query above capture', () => {
    const field = findSiblingQueryAssign(doc(insertLines), 6);
    expect(field?.rest.trim().startsWith('[[')).toBe(true);
    expect(field?.line).toBe(2);
  });

  it('resolves long-string captures', () => {
    const names = collectCaptureNameCompletions(doc(insertLines), 6, '');
    expect(names).toEqual(['body', 'name']);
  });

  it('filters by typed prefix', () => {
    const names = collectCaptureNameCompletions(doc(insertLines), 6, 'b');
    expect(names).toEqual(['body']);
  });

  it('resolves same-line inline query', () => {
    const lines = [
      '		c:replace({',
      '			query = "(declaration) @member",',
      '			capture = "mem',
    ];
    expect(collectCaptureNameCompletions(doc(lines), 2, 'mem')).toEqual([
      'member',
    ]);
  });

  it('returns scmPath for .scm sibling', () => {
    const lines = [
      '		c:remove({',
      '			query = "queries/foo.scm",',
      '			capture = "',
    ];
    expect(resolveSiblingQuery(doc(lines), 2)).toEqual({
      kind: 'scmPath',
      path: 'queries/foo.scm',
    });
  });

  it('reads .scm via callback', () => {
    const lines = [
      '		c:insert({',
      '			query = "body.scm",',
      '			capture = "',
      '			anchor = "end",',
    ];
    const names = collectCaptureNameCompletions(
      doc(lines),
      2,
      '',
      (p) => (p === 'body.scm' ? '(block) @body\n' : undefined)
    );
    expect(names).toEqual(['body']);
  });

  it('resolves query id via callback', () => {
    const lines = [
      '		c:insert({',
      '			query = "dart_queries.method_body",',
      '			capture = "',
    ];
    expect(resolveSiblingQuery(doc(lines), 2)).toEqual({
      kind: 'queryId',
      id: 'dart_queries.method_body',
    });
    const names = collectCaptureNameCompletions(
      doc(lines),
      2,
      '',
      undefined,
      (id) =>
        id === 'dart_queries.method_body'
          ? '(function_body (block) @body)'
          : undefined
    );
    expect(names).toEqual(['body']);
  });
});
