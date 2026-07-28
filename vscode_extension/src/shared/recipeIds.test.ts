import { describe, expect, it } from 'vitest';
import {
  collectRecipeIdCompletions,
  recipeIdCompletionContext,
} from './recipeIds';

const recipeIds = ['foo.bar.baz', 'foo.bab.bak', 'zap'];

describe('recipeIdCompletionContext', () => {
  it('extracts the partially typed id and its start offset', () => {
    expect(recipeIdCompletionContext('  recipe: foo.ba')).toEqual({
      typed: 'foo.ba',
      start: 10,
    });
    expect(recipeIdCompletionContext('id: foo.ba')).toEqual({
      typed: 'foo.ba',
      start: 4,
    });
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
