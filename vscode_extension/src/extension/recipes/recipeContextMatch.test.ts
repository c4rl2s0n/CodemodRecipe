import { describe, expect, it } from 'vitest';
import { matchRecipesToEditorContext } from './recipeContextMatch';
import type { RecipeSchema } from '../../shared';

function arg(
  name: string,
  opts: { from?: string | { template: string }; required?: boolean; defaultsTo?: string | null }
): RecipeSchema['args'][number] {
  return {
    name,
    abbr: null,
    help: null,
    required: opts.required ?? false,
    defaultsTo: opts.defaultsTo ?? null,
    inputKind: 'text',
    options: [],
    allowCustomValue: true,
    contextKey: null,
    from: opts.from ?? null,
  };
}

const recipes: RecipeSchema[] = [
  {
    id: 'with.file',
    name: 'With File',
    description: 'needs file',
    args: [arg('file', { from: 'file', required: true })],
  },
  {
    id: 'with.word',
    name: 'With Word',
    description: '',
    args: [
      arg('file', { from: 'file', required: true }),
      arg('name', { from: 'word', required: true }),
    ],
  },
  {
    id: 'no.from',
    name: 'No From',
    description: '',
    args: [arg('x', { required: true })],
  },
];

describe('matchRecipesToEditorContext', () => {
  it('returns recipes with any local prefill and marks completeness', () => {
    const matches = matchRecipesToEditorContext(recipes, {
      file: 'lib/a.dart',
    });
    expect(matches.map((m) => m.recipeId)).toEqual(['with.file', 'with.word']);
    expect(matches[0].complete).toBe(true);
    expect(matches[1].complete).toBe(false);
    expect(matches[1].args).toEqual({ file: 'lib/a.dart' });
  });

  it('returns empty when nothing matches', () => {
    expect(matchRecipesToEditorContext(recipes, {})).toEqual([]);
  });
});
