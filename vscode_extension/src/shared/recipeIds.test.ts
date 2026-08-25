import { describe, expect, it } from 'vitest';
import {
  collectRecipeIdCompletions,
  recipeIdCompletionContext,
} from './recipeIds';

const recipeIds = ['foo.bar.baz', 'foo.bab.bak', 'zap'];

describe('recipeIdCompletionContext', () => {
  it('extracts partial id after Luau id =', () => {
    expect(recipeIdCompletionContext('  id = "foo.ba')).toEqual({
      typed: 'foo.ba',
      start: 8,
    });
  });

  it('extracts partial id after c:run(', () => {
    expect(recipeIdCompletionContext('c:run("foo.ba')).toEqual({
      typed: 'foo.ba',
      start: 7,
    });
  });

  it('extracts after c:map(', () => {
    expect(recipeIdCompletionContext('local m = c:map("col')).toEqual({
      typed: 'col',
      start: 17,
    });
  });

  it('extracts after c:var and c:queryLib', () => {
    expect(recipeIdCompletionContext('c:var("d')?.typed).toBe('d');
    expect(recipeIdCompletionContext('c:queryLib("dart_')?.typed).toBe('dart_');
  });

  it('returns undefined outside id contexts', () => {
    expect(recipeIdCompletionContext('local x = "foo')).toBeUndefined();
  });
});

describe('collectRecipeIdCompletions', () => {
  it('suggests only the top-level segment first', () => {
    expect(collectRecipeIdCompletions(recipeIds, 'fo').map((item) => item.label)).toEqual([
      'foo',
    ]);
  });

  it('suggests only children after a dot', () => {
    expect(
      collectRecipeIdCompletions(recipeIds, 'foo.').map((item) => item.label)
    ).toEqual(['bab', 'bar']);
  });

  it('filters children by the current segment prefix', () => {
    expect(
      collectRecipeIdCompletions(recipeIds, 'foo.ba').map((item) => item.label)
    ).toEqual(['bab', 'bar']);
    expect(
      collectRecipeIdCompletions(recipeIds, 'foo.bar.').map((item) => item.label)
    ).toEqual(['baz']);
  });

  it('deduplicates shared prefixes', () => {
    expect(
      collectRecipeIdCompletions(['foo.bar.baz', 'foo.bar.bat'], 'foo.')
    ).toEqual([
      {
        label: 'bar',
        insertText: 'bar',
        fullPath: 'foo.bar',
        hasChildren: true,
      },
    ]);
  });
});
