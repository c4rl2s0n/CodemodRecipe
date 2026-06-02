import { spawn } from 'child_process';
import { ExtensionConfig } from '../config/extensionConfig';
import { HostDiscovery } from './hostDiscovery';
import {
  ApplyResponse,
  HostCommand,
  ListResponse,
  PreviewResponse,
  parseHostResponse,
} from './hostProtocol';
import { RecipeSchema, SelectionPayload } from '../types';

export class DartBridge {
  constructor(
    private readonly workspaceRoot: string,
    private readonly config: ExtensionConfig,
    private readonly hostDiscovery: HostDiscovery
  ) {}

  async list(): Promise<RecipeSchema[]> {
    const response = await this.send<ListResponse>({ command: 'list' });
    if (!response.ok || !response.recipes) {
      throw new Error(response.error ?? 'Failed to list recipes');
    }
    return response.recipes;
  }

  preview(recipe: string, args: Record<string, string>): Promise<PreviewResponse> {
    return this.send<PreviewResponse>({ command: 'preview', recipe, args });
  }

  apply(
    recipe: string,
    args: Record<string, string>,
    selection: SelectionPayload
  ): Promise<ApplyResponse> {
    return this.send<ApplyResponse>({
      command: 'apply',
      recipe,
      args,
      selection,
    });
  }

  private send<T>(command: HostCommand): Promise<T> {
    const entrypoint = this.hostDiscovery.resolveHostEntrypoint();
    const dart = this.config.dartPath;

    return new Promise<T>((resolve, reject) => {
      const child = spawn(dart, ['run', entrypoint], {
        cwd: this.workspaceRoot,
      });
      let stdout = '';
      let stderr = '';

      child.stdout.on('data', (chunk) => (stdout += chunk.toString()));
      child.stderr.on('data', (chunk) => (stderr += chunk.toString()));
      child.on('error', (err) => reject(err));
      child.on('close', (code) => {
        try {
          const response = parseHostResponse<T>(stdout);
          if (response === undefined) {
            reject(
              new Error(
                `Host produced no result (exit ${code}).\n${stderr || stdout}`
              )
            );
            return;
          }
          resolve(response);
        } catch (err) {
          reject(new Error(`Failed to parse host response: ${err}`));
        }
      });

      child.stdin.write(JSON.stringify(command));
      child.stdin.end();
    });
  }
}
