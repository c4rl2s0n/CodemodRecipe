import { describe, expect, it } from 'vitest';
import { buildHostBinaryArgs, hostSpawnConfigSignature } from './hostSpawnArgs';

describe('hostSpawnArgs', () => {
  it('builds Rust host binary argv', () => {
    expect(
      buildHostBinaryArgs({
        workspaceRoot: '/ws',
        codemodRoot: '.codemod',
      })
    ).toEqual([
      '--stdio-server',
      '--workspace-root',
      '/ws',
      '--codemod-root',
      '.codemod',
    ]);
  });

  it('signatures change when spawn config changes', () => {
    const a = hostSpawnConfigSignature({
      workspaceRoot: '/ws',
      codemodRoot: '.codemod',
    });
    const b = hostSpawnConfigSignature({
      workspaceRoot: '/ws',
      codemodRoot: 'custom',
    });
    expect(a).not.toEqual(b);
  });
});
