import * as fs from 'fs';
import * as path from 'path';
import { DEFAULT_HOST_CANDIDATES } from '../constants';
import { ExtensionConfig } from '../config/extensionConfig';

export class HostDiscovery {
  constructor(
    private readonly workspaceRoot: string,
    private readonly config: ExtensionConfig
  ) {}

  resolveHostEntrypoint(): string {
    const configured = this.config.hostEntrypoint.trim();
    if (configured.length > 0) {
      return path.isAbsolute(configured)
        ? configured
        : path.join(this.workspaceRoot, configured);
    }

    for (const candidate of DEFAULT_HOST_CANDIDATES) {
      const full = path.join(this.workspaceRoot, candidate);
      if (fs.existsSync(full)) {
        return full;
      }
    }

    throw new Error(
      'No codemod host entry point found. Set "codemodRecipe.hostEntrypoint" in settings.'
    );
  }
}
