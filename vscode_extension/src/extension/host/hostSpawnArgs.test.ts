import { describe, expect, it } from 'vitest';
import { buildHostSpawnArgs } from './hostSpawnArgs';

describe('buildHostSpawnArgs', () => {
  it('passes HostConfig flags to the generic stdio host', () => {
    expect(
      buildHostSpawnArgs({
        workspaceRoot: '/workspace',
        codemodRoot: '.codemod',
        emptyConstructorStyle: 'positional',
      })
    ).toEqual([
      'run',
      'bin/codemod_host.dart',
      '--stdio-server',
      '--workspace-root',
      '/workspace',
      '--codemod-root',
      '.codemod',
      '--empty-constructor-style',
      'positional',
    ]);
  });
});
