import { describe, expect, it } from 'vitest';
import {
  missingRequiredArgNames,
  prefillArgs,
  renderContextTemplate,
} from './recipeContextValues';
import type { RecipeSchema } from '../../shared';

const recipe: RecipeSchema = {
  id: 'demo',
  name: 'Demo',
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
      from: 'file',
    },
    {
      name: 'feature',
      abbr: null,
      help: null,
      required: true,
      defaultsTo: null,
      inputKind: 'text',
      options: [],
      allowCustomValue: true,
      contextKey: null,
      from: { template: '{{ fileDirname | basename }}' },
    },
    {
      name: 'legacy',
      abbr: null,
      help: null,
      required: false,
      defaultsTo: null,
      inputKind: 'text',
      options: [],
      allowCustomValue: true,
      contextKey: 'word',
    },
  ],
};

describe('prefillArgs', () => {
  it('resolves string from and contextKey builtins', () => {
    const args = prefillArgs(recipe, {
      file: 'lib/features/auth/view.dart',
      fileDirname: 'lib/features/auth',
      word: 'AuthView',
    });
    expect(args.file).toBe('lib/features/auth/view.dart');
    expect(args.feature).toBe('auth');
    expect(args.legacy).toBe('AuthView');
  });
});

describe('renderContextTemplate', () => {
  it('applies basename filter', () => {
    expect(
      renderContextTemplate('{{ fileDirname | basename }}', {
        fileDirname: 'a/b/c',
      })
    ).toBe('c');
  });
});

describe('missingRequiredArgNames', () => {
  it('applies defaultsTo before checking required', () => {
    const withDefault: RecipeSchema = {
      ...recipe,
      args: [
        {
          name: 'flag',
          abbr: null,
          help: null,
          required: true,
          defaultsTo: 'false',
          inputKind: 'text',
          options: [],
          allowCustomValue: true,
          contextKey: null,
        },
      ],
    };
    expect(missingRequiredArgNames(withDefault, {})).toEqual([]);
  });
});
