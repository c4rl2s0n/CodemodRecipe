import { ExtensionConfig } from '../config/extensionConfig';

export type HostSpawnConfig = {
  workspaceRoot: string;
  codemodRoot: string;
};

/** Stable signature used to decide when the persistent host must restart. */
export function hostSpawnConfigSignature(config: HostSpawnConfig): string {
  return JSON.stringify(config);
}

/** Builds argv for the Rust `codemod_host` binary (bundled or cargo-run). */
export function buildHostBinaryArgs(config: HostSpawnConfig): string[] {
  return [
    '--stdio-server',
    '--workspace-root',
    config.workspaceRoot,
    '--codemod-root',
    config.codemodRoot,
  ];
}

export function hostSpawnConfigFromExtension(
  extensionConfig: ExtensionConfig
): HostSpawnConfig {
  return {
    workspaceRoot: extensionConfig.workspaceRoot,
    codemodRoot: extensionConfig.codemodRoot,
  };
}
