import { ChildProcessWithoutNullStreams, spawn } from 'child_process';
import { ExtensionConfig } from '../config/extensionConfig';
import { HostDiscovery } from './hostDiscovery';
import {
  ApplyResponse,
  DescribeResponse,
  DiffResponse,
  extractHostResultFrame,
  HostCommand,
  ListResponse,
  PreviewResponse,
  parseHostResponse,
} from './hostProtocol';
import { RecipeSchema, SelectionPayload } from '../types';

type PendingRequest = {
  command: HostCommand['command'];
  startedAt: bigint;
  inputBytes: number;
  timeout: NodeJS.Timeout;
  resolve: (value: unknown) => void;
  reject: (error: Error) => void;
};

export class DartBridge {
  private static readonly perfPrefix = '[codemod-recipe/perf]';
  private static readonly requestTimeoutMs = 30000;

  private child: ChildProcessWithoutNullStreams | undefined;
  private pending: PendingRequest[] = [];
  private stdoutBuffer = '';
  private stderrBuffer = '';
  private queue = Promise.resolve();

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

  async describe(recipe: string): Promise<RecipeSchema> {
    const response = await this.send<DescribeResponse>({
      command: 'describe',
      recipe,
    });
    if (!response.ok || !response.recipe) {
      throw new Error(response.error ?? `Failed to describe recipe: ${recipe}`);
    }
    return response.recipe;
  }

  preview(recipe: string, args: Record<string, string>): Promise<PreviewResponse> {
    return this.send<PreviewResponse>({ command: 'preview', recipe, args });
  }

  async diff(
    recipe: string,
    args: Record<string, string>,
    path: string
  ): Promise<DiffResponse> {
    return this.send<DiffResponse>({ command: 'diff', recipe, args, path });
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

  dispose(): void {
    this.stopPersistentHost();
  }

  private async send<T>(command: HostCommand): Promise<T> {
    try {
      return await this.sendPersistent<T>(command);
    } catch (error) {
      this.logPerfWarning(
        `Persistent host failed for ${command.command}; retrying one-shot (${String(error)})`
      );
      this.stopPersistentHost();
      return this.sendOneShot<T>(command);
    }
  }

  private sendPersistent<T>(command: HostCommand): Promise<T> {
    return new Promise<T>((resolve, reject) => {
      this.queue = this.queue
        .then(async () => {
          const child = await this.ensurePersistentHost();
          this.stderrBuffer = '';
          const payload = `${JSON.stringify(command)}\n`;
          const inputBytes = Buffer.byteLength(payload, 'utf8');
          const timeout = setTimeout(() => {
            const index = this.pending.findIndex((request) => request.resolve === wrappedResolve);
            if (index >= 0) {
              const [request] = this.pending.splice(index, 1);
              request.reject(
                new Error(
                  `Timed out waiting for ${command.command} response (${DartBridge.requestTimeoutMs}ms)`
                )
              );
              this.stopPersistentHost();
            }
          }, DartBridge.requestTimeoutMs);
          const wrappedResolve = (value: unknown) => resolve(value as T);
          const wrappedReject = (err: Error) => reject(err);
          this.pending.push({
            command: command.command,
            startedAt: process.hrtime.bigint(),
            inputBytes,
            timeout,
            resolve: wrappedResolve,
            reject: wrappedReject,
          });
          child.stdin.write(payload);
        })
        .catch((err) => reject(err instanceof Error ? err : new Error(String(err))));
    });
  }

  private async ensurePersistentHost(): Promise<ChildProcessWithoutNullStreams> {
    if (this.child && !this.child.killed) {
      return this.child;
    }
    this.stdoutBuffer = '';
    this.stderrBuffer = '';
    const entrypoint = this.hostDiscovery.resolveHostEntrypoint();
    const dart = this.config.dartPath;
    return new Promise<ChildProcessWithoutNullStreams>((resolve, reject) => {
      const child = spawn(dart, ['run', entrypoint, '--stdio-server'], {
        cwd: this.workspaceRoot,
      });

      let startupResolved = false;
      child.stdout.on('data', (chunk) => {
        this.stdoutBuffer += chunk.toString();
        this.flushFrames();
        if (!startupResolved) {
          startupResolved = true;
          resolve(child);
        }
      });
      child.stderr.on('data', (chunk) => {
        this.stderrBuffer += chunk.toString();
      });
      child.on('error', (err) => {
        if (!startupResolved) {
          reject(err);
        }
        this.rejectPending(
          new Error(`Persistent host process error: ${err.message}`)
        );
        this.child = undefined;
      });
      child.on('close', (code) => {
        const error = new Error(
          `Persistent host exited with code ${code}\n${this.stderrBuffer.trim()}`
        );
        this.rejectPending(error);
        this.child = undefined;
      });

      const startupTimer = setTimeout(() => {
        if (!startupResolved) {
          reject(
            new Error(
              `Persistent host failed to start within ${DartBridge.requestTimeoutMs}ms`
            )
          );
          this.stopPersistentHost();
        }
      }, DartBridge.requestTimeoutMs);

      child.once('spawn', () => {
        this.child = child;
        if (!startupResolved) {
          startupResolved = true;
          clearTimeout(startupTimer);
          resolve(child);
        }
      });

      child.once('error', () => {
        clearTimeout(startupTimer);
      });
      child.once('close', () => {
        clearTimeout(startupTimer);
      });
    });
  }

  private flushFrames(): void {
    while (true) {
      const frame = extractHostResultFrame(this.stdoutBuffer);
      if (!frame) {
        return;
      }
      this.stdoutBuffer = frame.rest;
      const request = this.pending.shift();
      if (!request) {
        continue;
      }
      clearTimeout(request.timeout);
      try {
        const response = JSON.parse(frame.payload);
        const elapsedMs = Number(process.hrtime.bigint() - request.startedAt) / 1e6;
        this.logPerf({
          command: request.command,
          elapsedMs,
          inputBytes: request.inputBytes,
          outputBytes: Buffer.byteLength(frame.payload, 'utf8'),
        });
        request.resolve(response);
      } catch (err) {
        request.reject(new Error(`Failed to parse host response: ${err}`));
      }
    }
  }

  private sendOneShot<T>(command: HostCommand): Promise<T> {
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
              new Error(`Host produced no result (exit ${code}).\n${stderr || stdout}`)
            );
            return;
          }
          this.logPerf({ command: command.command, elapsedMs, inputBytes, outputBytes });
          resolve(response);
        } catch (err) {
          reject(new Error(`Failed to parse host response: ${err}`));
        }
      });

      child.stdin.write(payload);
      child.stdin.end();
    });
  }

  private stopPersistentHost(): void {
    if (this.child && !this.child.killed) {
      this.child.kill();
    }
    this.child = undefined;
  }

  private rejectPending(error: Error): void {
    for (const finalRequest of this.pending) {
      clearTimeout(finalRequest.timeout);
      finalRequest.reject(error);
    }
    this.pending = [];
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

  private logPerfWarning(message: string): void {
    if (!this.config.performanceLogging) {
      return;
    }
    console.warn(`${DartBridge.perfPrefix} ${message}`);
  }
}
