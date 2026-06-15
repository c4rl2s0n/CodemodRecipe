import { describe, expect, it } from 'vitest';
import { buildHostSpawnArgs } from './hostSpawnArgs';

describe('buildHostSpawnArgs', () => {
  it('passes HostConfig flags to the generic stdio host', () => {
    expect(
      buildHostSpawnArgs({
        workspaceRoot: '/workspace',
        entrypoint: '/workspace/bin/codemod_host.dart',
        recipesDirectory: '.codemod/recipes',
        templatesRoot: '.codemod/templates',
        emptyConstructorStyle: 'positional',
      })
    ).toEqual([
      'run',
      '/workspace/bin/codemod_host.dart',
      '--stdio-server',
      '--workspace-root',
      '/workspace',
      '--recipes-dir',
      '.codemod/recipes',
      '--templates-root',
      '.codemod/templates',
      '--empty-constructor-style',
      'positional',
    ]);
  });
});
