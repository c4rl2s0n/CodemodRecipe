import { describe, expect, it } from 'vitest';
import { argsKey, collectArgs, collectMissingRequiredArgs } from './args';
import type { RecipeSchema } from '../shared';

const sampleRecipe: RecipeSchema = {
  id: 'test',
  name: 'test',
  description: '',
  args: [
    {
      name: 'file',
      abbr: null,
      help: null,
      required: true,
      defaultsTo: null,
      inputKind: 'file',
      options: [],
      allowCustomValue: true,
      contextKey: null,
    },
    {
      name: 'optional',
      abbr: null,
      help: null,
      required: false,
      defaultsTo: 'x',
      inputKind: 'text',
      options: [],
      allowCustomValue: true,
      contextKey: null,
    },
  ],
};

describe('argsKey', () => {
  it('sorts keys for stable comparison', () => {
    expect(argsKey({ b: '2', a: '1' })).toBe(argsKey({ a: '1', b: '2' }));
  });
});

describe('collectMissingRequiredArgs', () => {
  it('returns missing required arg names', () => {
    expect(
      collectMissingRequiredArgs(sampleRecipe, { optional: 'y' })
    ).toEqual(['file']);
  });
});

describe('collectArgs', () => {
  it('omits empty values', () => {
    expect(
      collectArgs(sampleRecipe, { file: 'a.dart', optional: '' })
    ).toEqual({ file: 'a.dart' });
  });
});
