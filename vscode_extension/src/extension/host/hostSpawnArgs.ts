import { ExtensionConfig } from '../config/extensionConfig';

export type HostSpawnConfig = {
  workspaceRoot: string;
  entrypoint: string;
  recipesDirectory: string;
  templatesRoot: string;
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
    config.entrypoint,
    '--stdio-server',
    '--workspace-root',
    config.workspaceRoot,
    '--recipes-dir',
    config.recipesDirectory,
    '--templates-root',
    config.templatesRoot,
    '--empty-constructor-style',
    config.emptyConstructorStyle,
  ];
}

export function hostSpawnConfigFromExtension(
  workspaceRoot: string,
  entrypoint: string,
  extensionConfig: ExtensionConfig
): HostSpawnConfig {
  return {
    workspaceRoot,
    entrypoint,
    recipesDirectory: extensionConfig.recipesDirectory,
    templatesRoot: extensionConfig.templatesRoot,
    emptyConstructorStyle: extensionConfig.emptyConstructorStyle,
  };
}
