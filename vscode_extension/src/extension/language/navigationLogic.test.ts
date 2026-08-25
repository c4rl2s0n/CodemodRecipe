import { describe, expect, it } from 'vitest';
import {
  collectCodeLensDescriptors,
  resolveDefinitionTarget,
  resolveHoverContent,
} from './navigationLogic';
import type { RecipeSchema } from '../../shared';

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

function recipe(partial: Partial<RecipeSchema> & { id: string }): RecipeSchema {
  return {
    name: partial.name ?? partial.id,
    description: partial.description ?? '',
    args: partial.args ?? [],
    sourceFile: partial.sourceFile ?? `.codemod/recipes/${partial.id}.luau`,
    ...partial,
  };
}

describe('resolveDefinitionTarget', () => {
  it('resolves c:run recipe id to source file', () => {
    const line = '		c:run("foo.bar", {';
    const target = resolveDefinitionTarget(
      line,
      line.indexOf('foo') + 1,
      '',
      (id) => (id === 'foo.bar' ? recipe({ id: 'foo.bar' }) : undefined)
    );
    expect(target).toEqual({
      kind: 'recipe',
      id: 'foo.bar',
      sourceFile: '.codemod/recipes/foo.bar.luau',
    });
  });

  it('resolves template path', () => {
    const line = 'path = "templates/x.dart.template",';
    const target = resolveDefinitionTarget(
      line,
      line.indexOf('templates'),
      '',
      () => undefined
    );
    expect(target).toEqual({
      kind: 'template',
      path: 'templates/x.dart.template',
    });
  });
});

describe('resolveHoverContent', () => {
  it('documents host methods', () => {
    const line = '		c:insert({';
    const hover = resolveHoverContent(
      line,
      line.indexOf('insert') + 1,
      undefined,
      () => undefined,
      () => undefined
    );
    expect(hover?.kind).toBe('markdown');
    if (hover?.kind === 'markdown') {
      expect(hover.text).toContain('c:insert');
    }
  });

  it('documents recipe refs with args', () => {
    const line = '		c:run("child", {})';
    const hover = resolveHoverContent(
      line,
      line.indexOf('child') + 1,
      undefined,
      () =>
        recipe({
          id: 'child',
          name: 'Child',
          description: 'Does a thing',
          args: [
            {
              name: 'file',
              abbr: null,
              help: 'Target',
              required: true,
              defaultsTo: null,
              inputKind: 'file',
              options: [],
              allowCustomValue: false,
              contextKey: null,
            },
          ],
        }),
      () => undefined
    );
    expect(hover?.kind).toBe('markdown');
    if (hover?.kind === 'markdown') {
      expect(hover.text).toContain('Child');
      expect(hover.text).toContain('`file`');
      expect(hover.text).toContain('required');
    }
  });

  it('shows query source for queryLib ids', () => {
    const line = '		c:queryLib("dart_queries.method_body")';
    const hover = resolveHoverContent(
      line,
      line.indexOf('dart_queries') + 2,
      undefined,
      () => undefined,
      (id) => (id === 'dart_queries.method_body' ? '(block) @body' : undefined)
    );
    expect(hover?.kind).toBe('code');
    if (hover?.kind === 'code') {
      expect(hover.text).toContain('@body');
    }
  });

  it('documents module keywords', () => {
    const hover = resolveHoverContent(
      '	apply = function(c)',
      1,
      'apply',
      () => undefined,
      () => undefined
    );
    expect(hover?.kind).toBe('markdown');
    if (hover?.kind === 'markdown') {
      expect(hover.text).toContain('mutation');
    }
  });
});

describe('collectCodeLensDescriptors', () => {
  it('emits runner lenses on id and query tools on query', () => {
    const lines = [
      'return {',
      '	id = "demo.recipe",',
      '	apply = function(c)',
      '		c:insert({',
      '			query = [[(block) @b]],',
      '		})',
      '	end,',
      '}',
    ];
    const lenses = collectCodeLensDescriptors(doc(lines), {
      openInRecipeRunner: 'open',
      copyInvokeKeybinding: 'copy',
      assignToSlot: 'slot',
      queryToolsOpenFromRecipe: 'query',
    });
    expect(lenses.filter((l) => l.line === 1)).toHaveLength(3);
    expect(lenses.some((l) => l.command === 'query' && l.line === 4)).toBe(
      true
    );
    expect(lenses[0].arguments).toEqual(['demo.recipe']);
  });

  it('returns empty without top-level id', () => {
    expect(
      collectCodeLensDescriptors(doc(['-- comment']), {
        openInRecipeRunner: 'o',
        copyInvokeKeybinding: 'c',
        assignToSlot: 's',
        queryToolsOpenFromRecipe: 'q',
      })
    ).toEqual([]);
  });
});
