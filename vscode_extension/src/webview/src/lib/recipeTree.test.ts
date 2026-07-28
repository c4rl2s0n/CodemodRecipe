import { describe, expect, it } from 'vitest';
import type { RecipeSchema } from '../shared';
import {
  buildRecipeTree,
  recipeDisplayTitle,
  recipeGroupPath,
  recipeLeafId,
} from './recipeTree';

function recipe(id: string, name = id): RecipeSchema {
  return {
    id,
    name,
    description: '',
    args: [],
  };
}

describe('recipeLeafId', () => {
  it('returns the last dotted segment', () => {
    expect(recipeLeafId('rust.logging.insert')).toBe('insert');
    expect(recipeLeafId('single')).toBe('single');
  });
});

describe('recipeGroupPath', () => {
  it('returns every segment except the leaf id', () => {
    expect(recipeGroupPath('rust.logging.insert')).toEqual(['rust', 'logging']);
    expect(recipeGroupPath('single')).toEqual([]);
  });
});

describe('recipeDisplayTitle', () => {
  it('uses the leaf id when the fallback name equals the full id', () => {
    expect(recipeDisplayTitle(recipe('rust.logging.insert'))).toBe('insert');
  });

  it('preserves explicit human-readable names', () => {
    expect(recipeDisplayTitle(recipe('rust.logging.insert', 'Add Logging'))).toBe(
      'Add Logging'
    );
  });
});

describe('buildRecipeTree', () => {
  it('groups recipes by dotted id prefixes', () => {
    const tree = buildRecipeTree([
      recipe('rust.logging.insert'),
      recipe('rust.logging.remove'),
      recipe('dart.logging.insert'),
    ]);

    expect(tree.map((node) => node.label)).toEqual(['dart', 'rust']);
    expect(tree[0].children[0].label).toBe('logging');
    expect(tree[0].children[0].recipes.map((item) => item.id)).toEqual([
      'dart.logging.insert',
    ]);
    expect(tree[1].children[0].recipes.map((item) => item.id)).toEqual([
      'rust.logging.insert',
      'rust.logging.remove',
    ]);
  });

  it('keeps ids without dots in the ungrouped bucket', () => {
    const tree = buildRecipeTree([recipe('insert_log_line')]);
    expect(tree).toHaveLength(1);
    expect(tree[0].label).toBe('(ungrouped)');
    expect(tree[0].recipes.map((item) => item.id)).toEqual(['insert_log_line']);
  });
});
