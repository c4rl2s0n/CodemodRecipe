import { describe, expect, it } from 'vitest';
import {
  argsFieldCompletionContext,
  documentTopLevelId,
  documentTopLevelIdLine,
  findNearbyRunRecipeId,
  firstQueryAssignLine,
  inRunArgsTable,
  matchHostMethod,
  matchIdReference,
  matchTemplatePath,
  methodSpecContext,
  parseRecipeArgNames,
  recipeIdCompletionContext,
} from './luauContext';

function doc(lines: string[]) {
  return {
    lineCount: lines.length,
    lineAt(i: number) {
      return { text: lines[i] ?? '' };
    },
    getText() {
      return lines.join('\n');
    },
  };
}

describe('recipeIdCompletionContext', () => {
  it('detects id / run / map / var / queryLib', () => {
    expect(recipeIdCompletionContext('  id = "foo.')?.kind).toBe('id');
    expect(recipeIdCompletionContext('c:run("a.b')?.kind).toBe('run');
    expect(recipeIdCompletionContext('c:map("col')?.kind).toBe('map');
    expect(recipeIdCompletionContext('c:var("def')?.kind).toBe('var');
    expect(recipeIdCompletionContext('c:queryLib("dart')?.kind).toBe('queryLib');
  });
});

describe('matchIdReference / matchHostMethod / matchTemplatePath', () => {
  it('matches id under cursor', () => {
    const line = '  c:run("foo.bar", {';
    const start = line.indexOf('foo');
    expect(matchIdReference(line, start + 2)).toEqual({
      id: 'foo.bar',
      kind: 'run',
    });
    expect(matchIdReference(line, 0)).toBeUndefined();
  });

  it('matches host method name', () => {
    const line = '  c:insert({';
    expect(matchHostMethod(line, line.indexOf('insert') + 1)).toBe('insert');
  });

  it('matches template paths', () => {
    const line = `path = "templates/widget.dart.template",`;
    const idx = line.indexOf('templates');
    expect(matchTemplatePath(line, idx)).toBe(
      'templates/widget.dart.template'
    );
  });
});

describe('argsFieldCompletionContext', () => {
  it('detects c.args. prefix', () => {
    expect(argsFieldCompletionContext('  local x = c.args.fi')).toEqual({
      typed: 'fi',
      start: expect.any(Number),
    });
  });
});

describe('document id / query lines', () => {
  const lines = [
    'return {',
    '	id = "demo.recipe",',
    '	args = {},',
    '	apply = function(c)',
    '		c:insert({',
    '			query = [[(block) @body]],',
    '			capture = "body",',
    '		})',
    '	end,',
    '}',
  ];

  it('finds top-level id and first query', () => {
    const d = doc(lines);
    expect(documentTopLevelId(d)).toBe('demo.recipe');
    expect(documentTopLevelIdLine(d)).toBe(1);
    expect(firstQueryAssignLine(d)).toBe(5);
  });
});

describe('c:run args table', () => {
  const lines = [
    '		c:run("child.recipe", {',
    '			fi',
    '		})',
  ];

  it('detects nested run args and child id', () => {
    const d = doc(lines);
    expect(findNearbyRunRecipeId(d, 1)).toBe('child.recipe');
    expect(inRunArgsTable(d, 1, lines[1].length)).toBe(true);
  });
});

describe('methodSpecContext value enums', () => {
  it('detects anchor value completion', () => {
    const lines = [
      '		c:insert({',
      '			path = c.args.file,',
      '			anchor = "st',
    ];
    const d = doc(lines);
    const ctx = methodSpecContext(d, 2, lines[2].length);
    expect(ctx?.method).toBe('insert');
    expect(ctx?.valueField).toBe('anchor');
    expect(ctx?.typedValue).toBe('st');
  });
});

describe('parseRecipeArgNames', () => {
  it('returns empty when no args table', () => {
    expect(parseRecipeArgNames('return { id = "x", apply = function(c) end }')).toEqual(
      []
    );
  });
});
