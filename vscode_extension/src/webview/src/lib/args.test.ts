import { describe, expect, it } from 'vitest';
import {
  argsKey,
  collectArgs,
  collectMissingRequiredArgs,
  mergeArgValuesOnRefresh,
} from './args';
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

  it('treats an empty list as present', () => {
    const listRecipe: RecipeSchema = {
      ...sampleRecipe,
      args: [
        {
          ...sampleRecipe.args[0],
          name: 'properties',
          required: true,
          inputKind: ['ClassProperty'],
        },
      ],
    };
    expect(collectMissingRequiredArgs(listRecipe, { properties: [] })).toEqual(
      []
    );
  });
});

describe('collectArgs', () => {
  it('omits empty values', () => {
    expect(
      collectArgs(sampleRecipe, { file: 'a.dart', optional: '' })
    ).toEqual({ file: 'a.dart' });
  });

  it('sanitizes blank list rows and omits empty lists', () => {
    const listRecipe: RecipeSchema = {
      ...sampleRecipe,
      args: [
        {
          ...sampleRecipe.args[0],
          name: 'exports',
          required: false,
          inputKind: ['text'],
        },
      ],
    };
    expect(
      collectArgs(listRecipe, { exports: ['foo.dart', '', '  ', 'bar.dart'] })
    ).toEqual({ exports: ['foo.dart', 'bar.dart'] });
    expect(collectArgs(listRecipe, { exports: ['', '  '] })).toEqual({});
  });
});

describe('mergeArgValuesOnRefresh', () => {
  it('keeps values for surviving args and drops removed keys', () => {
    const updatedRecipe: RecipeSchema = {
      ...sampleRecipe,
      args: [
        sampleRecipe.args[0],
        {
          name: 'added',
          abbr: null,
          help: null,
          required: false,
          defaultsTo: 'new-default',
          inputKind: 'text',
          options: [],
          allowCustomValue: true,
          contextKey: null,
        },
      ],
    };
    expect(
      mergeArgValuesOnRefresh(updatedRecipe, {
        file: 'lib/a.dart',
        optional: 'keep-me',
        removed: 'gone',
      })
    ).toEqual({
      file: 'lib/a.dart',
      added: 'new-default',
    });
  });
});
