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
  private static readonly perfPrefix = '[codemod-recipe/perf]';

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
    const payload = JSON.stringify(command);
    const inputBytes = Buffer.byteLength(payload, 'utf8');
    const start = process.hrtime.bigint();

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
        const elapsedMs = Number(process.hrtime.bigint() - start) / 1e6;
        const outputBytes = Buffer.byteLength(stdout, 'utf8');
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
          this.logPerf({
            command: command.command,
            elapsedMs,
            inputBytes,
            outputBytes,
          });
          resolve(response);
        } catch (err) {
          reject(new Error(`Failed to parse host response: ${err}`));
        }
      });

      child.stdin.write(payload);
      child.stdin.end();
    });
  }

  private logPerf(metrics: {
    command: HostCommand['command'];
    elapsedMs: number;
    inputBytes: number;
    outputBytes: number;
  }): void {
    if (!this.config.performanceLogging) {
      return;
    }
    console.info(
      `${DartBridge.perfPrefix} command=${metrics.command} elapsedMs=${metrics.elapsedMs.toFixed(
        1
      )} inputBytes=${metrics.inputBytes} outputBytes=${metrics.outputBytes}`
    );
  }
}
