import * as path from 'path';
import { ExtensionConfig } from '../config/extensionConfig';

/// Host discovery for the new codemodRoot-based approach.
/// We always use the bundled bin/codemod_host.dart, so no discovery is needed.
export class HostDiscovery {
  constructor(
    private readonly workspaceRoot: string,
    private readonly config: ExtensionConfig
  ) {}

  /// Returns the entrypoint path for the bundled host.
  /// In the new architecture, we always use bin/codemod_host.dart from the package.
  resolveHostEntrypoint(): string {
    // We always use the standard entrypoint from codemod_recipe package
    return path.join(this.workspaceRoot, 'bin', 'codemod_host.dart');
  }

  /// Returns the codemod root directory path.
  getCodemodRootPath(): string {
    return path.join(this.workspaceRoot, this.config.codemodRoot);
  }
}
