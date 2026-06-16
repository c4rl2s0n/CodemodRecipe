import { ExtensionConfig } from '../config/extensionConfig';

export type HostSpawnConfig = {
  workspaceRoot: string;
  codemodRoot: string;
  emptyConstructorStyle: 'named' | 'positional';
};

/** Stable signature used to decide when the persistent host must restart. */
export function hostSpawnConfigSignature(config: HostSpawnConfig): string {
  return JSON.stringify(config);
}

/** Builds `dart run …` argv for the persistent stdio host. */
export function buildHostSpawnArgs(config: HostSpawnConfig): string[] {
  return [
    'run',
    'bin/codemod_host.dart',
    '--stdio-server',
    '--workspace-root',
    config.workspaceRoot,
    '--codemod-root',
    config.codemodRoot,
    '--empty-constructor-style',
    config.emptyConstructorStyle,
  ];
}

export function hostSpawnConfigFromExtension(
  workspaceRoot: string,
  extensionConfig: ExtensionConfig
): HostSpawnConfig {
  return {
    workspaceRoot,
    codemodRoot: extensionConfig.codemodRoot,
    emptyConstructorStyle: extensionConfig.emptyConstructorStyle,
  };
}
