import { describe, expect, it } from 'vitest';
import {
  collectCompletionSuggestions,
  type CompletionCatalog,
} from './completionLogic';
import type { RecipeSchema, TypeSchema } from '../../shared';

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

function emptyArg(name: string, required = false): RecipeSchema['args'][0] {
  return {
    name,
    abbr: null,
    help: null,
    required,
    defaultsTo: null,
    inputKind: 'text',
    options: [],
    allowCustomValue: false,
    contextKey: null,
  };
}

const catalog: CompletionCatalog = {
  recipeIds: ['foo.bar', 'foo.baz', 'other'],
  mapIds: ['column_type'],
  varIds: ['defaults'],
  queryIds: ['dart_queries.method_body'],
  languageIds: ['dart', 'rust'],
  typeIds: ['ClassProperty'],
  types: [
    {
      id: 'ClassProperty',
      fields: [emptyArg('name', true), emptyArg('type')],
    } satisfies TypeSchema,
  ],
  getQuerySource: (id) =>
    id === 'dart_queries.method_body'
      ? '(function_body (block) @body)'
      : undefined,
  describeRecipe: (id) =>
    id === 'child.recipe'
      ? {
          id: 'child.recipe',
          name: 'child',
          description: '',
          args: [emptyArg('file', true), emptyArg('className')],
        }
      : undefined,
};

describe('collectCompletionSuggestions', () => {
  it('suggests c: methods', () => {
    const lines = ['		c:ins'];
    const items = collectCompletionSuggestions({
      line: lines[0],
      before: lines[0],
      lineNumber: 0,
      character: lines[0].length,
      document: doc(lines),
      catalog,
    });
    expect(items?.map((i) => i.label)).toContain('insert');
    // Full "args" field only when typed prefix matches "args"
    const argsLine = ['		c:a'];
    const argsItems = collectCompletionSuggestions({
      line: argsLine[0],
      before: argsLine[0],
      lineNumber: 0,
      character: argsLine[0].length,
      document: doc(argsLine),
      catalog,
    });
    expect(argsItems?.some((i) => i.label === 'args')).toBe(true);
  });

  it('suggests hierarchical recipe ids for c:run', () => {
    const lines = ['		c:run("foo.'];
    const items = collectCompletionSuggestions({
      line: lines[0],
      before: lines[0],
      lineNumber: 0,
      character: lines[0].length,
      document: doc(lines),
      catalog,
    });
    expect(items?.map((i) => i.label).sort()).toEqual(['bar', 'baz']);
  });

  it('suggests map ids for c:map', () => {
    const lines = ['		c:map("col'];
    const items = collectCompletionSuggestions({
      line: lines[0],
      before: lines[0],
      lineNumber: 0,
      character: lines[0].length,
      document: doc(lines),
      catalog,
    });
    expect(items?.map((i) => i.label)).toEqual(['column_type']);
  });

  it('suggests remaining insert spec keys', () => {
    const lines = [
      '		c:insert({',
      '			path = c.args.file,',
      '			que',
    ];
    const items = collectCompletionSuggestions({
      line: lines[2],
      before: lines[2],
      lineNumber: 2,
      character: lines[2].length,
      document: doc(lines),
      catalog,
    });
    expect(items?.map((i) => i.label)).toContain('query');
    expect(items?.every((i) => i.label !== 'path')).toBe(true);
  });

  it('suggests anchor enum values', () => {
    const lines = [
      '		c:insert({',
      '			path = c.args.file,',
      '			anchor = "s',
    ];
    const items = collectCompletionSuggestions({
      line: lines[2],
      before: lines[2],
      lineNumber: 2,
      character: lines[2].length,
      document: doc(lines),
      catalog,
    });
    expect(items?.map((i) => i.label)).toEqual(['start']);
  });

  it('suggests language ids', () => {
    const lines = [
      '		c:insert({',
      '			language = "da',
    ];
    const items = collectCompletionSuggestions({
      line: lines[1],
      before: lines[1],
      lineNumber: 1,
      character: lines[1].length,
      document: doc(lines),
      catalog,
    });
    expect(items?.map((i) => i.label)).toEqual(['dart']);
  });

  it('suggests captures from sibling long-string query', () => {
    const lines = [
      '		c:insert({',
      '			query = [[',
      '(block) @body',
      '(identifier) @name',
      ']],',
      '			capture = "',
    ];
    const items = collectCompletionSuggestions({
      line: lines[5],
      before: lines[5],
      lineNumber: 5,
      character: lines[5].length,
      document: doc(lines),
      catalog,
    });
    expect(items?.map((i) => i.label)).toEqual(['body', 'name']);
  });

  it('suggests captures from registered query id', () => {
    const lines = [
      '		c:insert({',
      '			query = "dart_queries.method_body",',
      '			capture = "',
    ];
    const items = collectCompletionSuggestions({
      line: lines[2],
      before: lines[2],
      lineNumber: 2,
      character: lines[2].length,
      document: doc(lines),
      catalog,
    });
    expect(items?.map((i) => i.label)).toEqual(['body']);
  });

  it('suggests c.args fields from recipe source', () => {
    const lines = [
      'return {',
      '	id = "x",',
      '	args = { { name = "file" }, { name = "className" } },',
      '	apply = function(c)',
      '		local p = c.args.cl',
      '	end,',
      '}',
    ];
    const items = collectCompletionSuggestions({
      line: lines[4],
      before: lines[4],
      lineNumber: 4,
      character: lines[4].length,
      document: doc(lines),
      catalog,
    });
    expect(items?.map((i) => i.label)).toEqual(['className']);
  });

  it('suggests nested c:run args via describeRecipe', () => {
    const lines = [
      '		c:run("child.recipe", {',
      '			fi',
    ];
    const items = collectCompletionSuggestions({
      line: lines[1],
      before: lines[1],
      lineNumber: 1,
      character: lines[1].length,
      document: doc(lines),
      catalog,
    });
    expect(items?.map((i) => i.label)).toContain('file');
  });

  it('suggests inputKind builtins and named types', () => {
    const lines = ['			inputKind = "fi'];
    const items = collectCompletionSuggestions({
      line: lines[0],
      before: lines[0],
      lineNumber: 0,
      character: lines[0].length,
      document: doc(lines),
      catalog,
    });
    expect(items?.map((i) => i.label)).toContain('file');
  });

  it('suggests module keys at shallow indent', () => {
    const lines = ['	des'];
    const items = collectCompletionSuggestions({
      line: lines[0],
      before: lines[0],
      lineNumber: 0,
      character: lines[0].length,
      document: doc(lines),
      catalog,
    });
    expect(items?.map((i) => i.label)).toContain('description');
  });

  it('suggests jinja arg and map refs', () => {
    const lines = [
      'return {',
      '	args = { { name = "file" } },',
      '	apply = function(c)',
      '		c:postExec({ command = "{{ fi',
    ];
    const items = collectCompletionSuggestions({
      line: lines[3],
      before: lines[3],
      lineNumber: 3,
      character: lines[3].length,
      document: doc(lines),
      catalog,
    });
    expect(items?.map((i) => i.label)).toEqual(
      expect.arrayContaining(['file', 'var.defaults', 'map.column_type'])
    );
  });
});
