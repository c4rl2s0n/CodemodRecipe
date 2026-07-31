import { describe, expect, it } from 'vitest';
import {
  buildUriContextValues,
  prefillArgsFromUriClick,
  toWorkspaceRelativePath,
} from './recipeUriContext';

describe('uri context helpers', () => {
  it('toWorkspaceRelativePath keeps paths under root relative', () => {
    expect(toWorkspaceRelativePath('/ws', '/ws/lib/foo.dart')).toBe(
      'lib/foo.dart'
    );
  });

  it('buildUriContextValues sets path and file/directory builtins', () => {
    expect(buildUriContextValues('lib/a.dart', 'file').values).toMatchObject({
      path: 'lib/a.dart',
      file: 'lib/a.dart',
      directory: 'lib',
      fileStem: 'a',
      fileExt: 'dart',
    });
    expect(
      buildUriContextValues('lib/features', 'folder').values
    ).toMatchObject({
      path: 'lib/features',
      directory: 'lib/features',
      fileDirname: 'lib/features',
    });
  });

  it('prefillArgsFromUriClick fills first matching inputKind', () => {
    const recipe = {
      args: [
        { name: 'name', inputKind: 'text' },
        { name: 'directory', inputKind: 'directory' },
        { name: 'otherDir', inputKind: 'directory' },
      ],
    };
    expect(prefillArgsFromUriClick(recipe, 'folder', 'lib/features')).toEqual({
      directory: 'lib/features',
    });
    expect(prefillArgsFromUriClick(recipe, 'file', 'lib/a.dart')).toEqual({});
  });
});
