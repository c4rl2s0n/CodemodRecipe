import * as fs from 'fs';
import * as os from 'os';
import * as path from 'path';
import { ChildProcessWithoutNullStreams, spawn } from 'child_process';
import * as vscode from 'vscode';
import { ExtensionConfig } from '../config/extensionConfig';
import {
  buildHostBinaryArgs,
  hostSpawnConfigFromExtension,
  hostSpawnConfigSignature,
  type HostSpawnConfig,
} from './hostSpawnArgs';
import {
  ApplyResponse,
  DescribeResponse,
  DiffResponse,
  extractHostResultFrame,
  HostCommand,
  PreviewResponse,
  RecipeCatalogResponse,
  ValidateResponse,
  parseHostResponse,
} from './hostProtocol';
import type { RecipeSchema, SelectionPayload, BootstrapResponse, DeriveArgsRequest, DeriveArgsResponse } from '../../shared';
import type { RecipeLoadResult } from '../recipes/recipeRepository';

type PendingRequest = {
  command: HostCommand['command'];
  startedAt: bigint;
  inputBytes: number;
  timeout: NodeJS.Timeout;
  resolve: (value: unknown) => void;
  reject: (error: Error) => void;
};

const HOST_BUILD_HINT =
  'Build the host with `vscode_extension/build.sh` (or `cargo build -p codemod_recipe_host --bin codemod_host` from `rust/`).';

export class HostBridge {
  private static readonly perfPrefix = '[codemod-recipe/perf]';
  private static readonly requestTimeoutMs = 30000;

  private child: ChildProcessWithoutNullStreams | undefined;
  private hostConfigSignature: string | undefined;
  private pending: PendingRequest[] = [];
  private stdoutBuffer = '';
  private stderrBuffer = '';
  private queue = Promise.resolve();

  constructor(
    private readonly config: ExtensionConfig,
    private readonly extensionUri: vscode.Uri
  ) {}

  private get workspaceRoot(): string {
    return this.config.workspaceRoot;
  }

  /**
   * Returns the path to the bundled codemod_host binary for the current platform.
   */
  private getBinaryPath(): string {
    const platform = os.platform();
    const binDir = path.join(this.extensionUri.fsPath, 'bin');

    switch (platform) {
      case 'win32':
        return path.join(binDir, 'codemod_host.exe');
      case 'darwin':
      case 'linux':
      default:
        return path.join(binDir, 'codemod_host');
    }
  }

  private hasBundledBinary(): boolean {
    try {
      return fs.existsSync(this.getBinaryPath());
    } catch {
      return false;
    }
  }

  /**
   * Returns the command and arguments to spawn the Rust host process.
   * Prefer the bundled binary; fall back to `cargo run` only when developing
   * inside a checkout that contains `rust/Cargo.toml`.
   */
  private getSpawnCommand(): { command: string; args: string[] } {
    if (this.hasBundledBinary()) {
      return {
        command: this.getBinaryPath(),
        args: buildHostBinaryArgs(this.currentHostSpawnConfig()),
      };
    }

    const hostConfig = this.currentHostSpawnConfig();
    const manifestPath = path.join(this.workspaceRoot, 'rust', 'Cargo.toml');
    if (!fs.existsSync(manifestPath)) {
      throw new Error(
        `Bundled codemod_host not found at ${this.getBinaryPath()}, and no ${manifestPath} for cargo fallback. ${HOST_BUILD_HINT}`
      );
    }
    return {
      command: 'cargo',
      args: [
        'run',
        '-q',
        '--manifest-path',
        manifestPath,
        '-p',
        'codemod_recipe_host',
        '--bin',
        'codemod_host',
        '--',
        ...buildHostBinaryArgs(hostConfig),
      ],
    };
  }

  async listRecipes(): Promise<RecipeLoadResult> {
    const response = await this.send<RecipeCatalogResponse>({ command: 'list' });
    return this.parseRecipeLoadResponse(response, 'list');
  }

  async reloadRecipes(): Promise<RecipeLoadResult> {
    const response = await this.send<RecipeCatalogResponse>({ command: 'reload' });
    return this.parseRecipeLoadResponse(response, 'reload');
  }

  async validateRecipes(recipeId?: string): Promise<ValidateResponse> {
    return this.send<ValidateResponse>({
      command: 'validate',
      ...(recipeId ? { recipe: recipeId } : {}),
    });
  }

  async ensureHost(): Promise<void> {
    await this.ensurePersistentHost();
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

  preview(
    recipe: string,
    args: Record<string, string>,
    snippetLines?: number
  ): Promise<PreviewResponse> {
    return this.send<PreviewResponse>({
      command: 'preview',
      recipe,
      args,
      snippetLines,
    });
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
    selection: SelectionPayload,
    previewToken: string
  ): Promise<ApplyResponse> {
    return this.send<ApplyResponse>({
      command: 'apply',
      recipe,
      args,
      selection,
      previewToken,
    });
  }

  deriveArgs(request: DeriveArgsRequest): Promise<DeriveArgsResponse> {
    return this.send<DeriveArgsResponse>({
      command: 'deriveArgs',
      recipe: request.recipe,
      source: request.source,
      language: request.language,
      path: request.path,
      cursorOffset: request.cursorOffset,
      selectionStart: request.selectionStart,
      selectionEnd: request.selectionEnd,
      context: request.context,
    });
  }

  async bootstrapProject(
    force = false,
    options?: { editPolicy?: string; companions?: string[] }
  ): Promise<BootstrapResponse> {
    return this.send<BootstrapResponse>({
      command: 'bootstrap',
      force,
      ...(options?.editPolicy ? { editPolicy: options.editPolicy } : {}),
      ...(options?.companions ? { companions: options.companions } : {}),
    });
  }

  dispose(): void {
    this.stopPersistentHost();
  }

  private parseRecipeLoadResponse(
    response: RecipeCatalogResponse,
    command: 'list' | 'reload'
  ): RecipeLoadResult {
    if (!response.ok) {
      throw new Error(response.error ?? `Failed to ${command} recipes`);
    }
    return {
      recipes: response.recipes ?? [],
      diagnostics: response.diagnostics ?? [],
      mapIds: response.mapIds ?? [],
      varIds: response.varIds ?? [],
      languageIds: response.languageIds ?? [],
    };
  }

  private currentHostSpawnConfig(): HostSpawnConfig {
    return hostSpawnConfigFromExtension(this.config);
  }

  private ensureHostConfigCurrent(): void {
    const signature = hostSpawnConfigSignature(this.currentHostSpawnConfig());
    if (this.hostConfigSignature !== undefined && this.hostConfigSignature !== signature) {
      this.stopPersistentHost();
    }
    this.hostConfigSignature = signature;
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
                  `Timed out waiting for ${command.command} response (${HostBridge.requestTimeoutMs}ms)`
                )
              );
              this.stopPersistentHost();
            }
          }, HostBridge.requestTimeoutMs);
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
    this.ensureHostConfigCurrent();
    if (this.child && !this.child.killed) {
      return this.child;
    }
    this.stdoutBuffer = '';
    this.stderrBuffer = '';

    const { command, args } = this.getSpawnCommand();
    const usingCargo = command === 'cargo';
    return new Promise<ChildProcessWithoutNullStreams>((resolve, reject) => {
      const child = spawn(command, args, {
        cwd: this.workspaceRoot,
      });

      let startupResolved = false;
      const failStartup = (err: Error) => {
        if (!startupResolved) {
          startupResolved = true;
          reject(err);
        }
      };

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
        const hint = usingCargo
          ? ` Failed to spawn cargo (${err.message}). Ensure Rust/cargo is on PATH, or ${HOST_BUILD_HINT}`
          : ` Failed to spawn host binary (${err.message}). ${HOST_BUILD_HINT}`;
        failStartup(new Error(hint.trim()));
        this.rejectPending(
          new Error(`Persistent host process error: ${err.message}`)
        );
        this.child = undefined;
      });
      child.on('close', (code) => {
        const stderr = this.stderrBuffer.trim();
        const hint =
          usingCargo || code !== 0
            ? `\n${HOST_BUILD_HINT}`
            : '';
        const error = new Error(
          `Persistent host exited with code ${code}${stderr ? `\n${stderr}` : ''}${hint}`
        );
        failStartup(error);
        this.rejectPending(error);
        this.child = undefined;
      });

      const startupTimer = setTimeout(() => {
        if (!startupResolved) {
          failStartup(
            new Error(
              `Persistent host failed to start within ${HostBridge.requestTimeoutMs}ms.\n${HOST_BUILD_HINT}`
            )
          );
          this.stopPersistentHost();
        }
      }, HostBridge.requestTimeoutMs);

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
    const { command: cmd, args: spawnArgs } = this.getSpawnCommand();
    const payload = JSON.stringify(command);
    const inputBytes = Buffer.byteLength(payload, 'utf8');
    const start = process.hrtime.bigint();

    return new Promise<T>((resolve, reject) => {
      const child = spawn(cmd, spawnArgs, {
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

      child.stdin.write(`${payload}\n`);
      child.stdin.end();
    });
  }

  private stopPersistentHost(): void {
    if (this.child && !this.child.killed) {
      this.child.kill();
    }
    this.child = undefined;
    this.hostConfigSignature = undefined;
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
      `${HostBridge.perfPrefix} command=${metrics.command} elapsedMs=${metrics.elapsedMs.toFixed(
        1
      )} inputBytes=${metrics.inputBytes} outputBytes=${metrics.outputBytes}`
    );
  }

  private logPerfWarning(message: string): void {
    if (!this.config.performanceLogging) {
      return;
    }
    console.warn(`${HostBridge.perfPrefix} ${message}`);
  }
}
