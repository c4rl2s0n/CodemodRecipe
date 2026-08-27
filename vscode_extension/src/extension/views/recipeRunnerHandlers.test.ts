import { beforeEach, describe, expect, it, vi } from 'vitest';
import {
  EXTENSION_TO_WEBVIEW,
  RUNNER_SESSION_PHASE,
  WEBVIEW_TO_EXTENSION,
  type RecipeSchema,
} from '../../shared';
import { RecipeRunnerState } from './recipeRunnerState';
import {
  handleWebviewMessage,
  type RecipeRunnerHandlerHost,
} from './recipeRunnerHandlers';

const recipe: RecipeSchema = {
  id: 'r1',
  name: 'R1',
  description: '',
  args: [],
  sourceFile: '.codemod/recipes/r1.luau',
};

function createHost(overrides: Partial<RecipeRunnerHandlerHost> = {}) {
  const state = new RecipeRunnerState();
  state.currentRecipe = recipe;
  state.recipes = [recipe];
  let previewInFlight = false;
  let previewGeneration = 0;
  const messages: unknown[] = [];

  const host: RecipeRunnerHandlerHost = {
    workspaceRoot: '/tmp/ws',
    state,
    bridge: {
      preview: vi.fn(async () => ({
        ok: true,
        files: [],
        previewToken: 'tok',
        transcript: [],
      })),
      apply: vi.fn(),
      diff: vi.fn(),
      describe: vi.fn(),
      cancelInFlight: vi.fn(),
    } as unknown as RecipeRunnerHandlerHost['bridge'],
    config: { previewSnippetLines: 3 } as RecipeRunnerHandlerHost['config'],
    diffProvider: { store: vi.fn(), clear: vi.fn() } as unknown as RecipeRunnerHandlerHost['diffProvider'],
    get previewInFlight() {
      return previewInFlight;
    },
    setPreviewInFlight(value: boolean) {
      previewInFlight = value;
    },
    get previewGeneration() {
      return previewGeneration;
    },
    beginPreviewGeneration() {
      previewGeneration += 1;
      return previewGeneration;
    },
    scaffoldHandler: undefined,
    postState: vi.fn(),
    postMessage(message: unknown) {
      messages.push(message);
    },
    run: vi.fn(),
    ensureRecipeDetails: async (r) => r,
    argsKey: (args) => JSON.stringify(args),
    recipeRepository: undefined,
    resetSession() {
      previewInFlight = false;
      previewGeneration += 1;
      state.beginSession();
    },
    publishRuntimeDiagnostics: vi.fn(),
    ...overrides,
  };

  return { host, state, messages, getPreviewInFlight: () => previewInFlight };
}

vi.mock('vscode', () => ({
  window: {
    showOpenDialog: vi.fn(),
    showInformationMessage: vi.fn(),
    showErrorMessage: vi.fn(),
  },
  Uri: { file: (p: string) => ({ fsPath: p }) },
  commands: { executeCommand: vi.fn() },
}));

describe('recipeRunnerHandlers ACK protocol', () => {
  beforeEach(() => {
    vi.resetModules();
  });

  it('posts cancelled filePicked when dialog is dismissed', async () => {
    const vscode = await import('vscode');
    vi.mocked(vscode.window.showOpenDialog).mockResolvedValue(undefined);

    const { host, messages } = createHost();
    await handleWebviewMessage(host, {
      type: WEBVIEW_TO_EXTENSION.pickFile,
      arg: 'path',
      requestId: 7,
    });

    expect(messages).toContainEqual({
      type: EXTENSION_TO_WEBVIEW.filePicked,
      arg: 'path',
      requestId: 7,
      cancelled: true,
    });
    expect(host.state.sessionPhase).toBe(RUNNER_SESSION_PHASE.idle);
  });

  it('supersedes in-flight preview via cancelInFlight and runs the new preview', async () => {
    const cancelInFlight = vi.fn();
    const preview = vi.fn(async () => ({
      ok: true,
      files: [],
      previewToken: 'tok',
      transcript: [],
    }));
    const { host, messages } = createHost({
      bridge: {
        preview,
        cancelInFlight,
        apply: vi.fn(),
        diff: vi.fn(),
        describe: vi.fn(),
      } as unknown as RecipeRunnerHandlerHost['bridge'],
    });
    host.setPreviewInFlight(true);

    await handleWebviewMessage(host, {
      type: WEBVIEW_TO_EXTENSION.preview,
      args: { x: '1' },
      requestId: 9,
    });
    // Preview is fire-and-forget; wait for the async handler to settle.
    await vi.waitFor(() => expect(preview).toHaveBeenCalled());

    expect(cancelInFlight).toHaveBeenCalledWith('Preview superseded');
    expect(messages).toContainEqual({
      type: EXTENSION_TO_WEBVIEW.previewState,
      inFlight: true,
      requestId: 9,
    });
    expect(messages).toContainEqual({
      type: EXTENSION_TO_WEBVIEW.previewResult,
      files: [],
      transcript: [],
      applyMayDiverge: false,
      requestId: 9,
      argsKey: '{"x":"1"}',
    });
  });

  it('posts runtime diagnostics when preview fails', async () => {
    const publishRuntimeDiagnostics = vi.fn();
    const { host, messages } = createHost({
      publishRuntimeDiagnostics,
      bridge: {
        preview: vi.fn(async () => ({
          ok: false,
          error: 'Lua apply boom',
          diagnostics: [
            {
              severity: 'error',
              code: 'E_RECIPE_RUNTIME',
              message: 'Lua apply boom',
            },
          ],
        })),
        cancelInFlight: vi.fn(),
        apply: vi.fn(),
        diff: vi.fn(),
        describe: vi.fn(),
      } as unknown as RecipeRunnerHandlerHost['bridge'],
    });

    await handleWebviewMessage(host, {
      type: WEBVIEW_TO_EXTENSION.preview,
      args: {},
      requestId: 4,
    });
    await vi.waitFor(() =>
      expect(messages.some((m) => (m as { type?: string }).type === EXTENSION_TO_WEBVIEW.error)).toBe(
        true
      )
    );

    expect(publishRuntimeDiagnostics).toHaveBeenCalled();
    const err = messages.find(
      (m) => (m as { type?: string }).type === EXTENSION_TO_WEBVIEW.error
    ) as {
      message: string;
      diagnostics: { code: string }[];
    };
    expect(err.message).toBe('Lua apply boom');
    expect(err.diagnostics[0]?.code).toBe('E_RECIPE_RUNTIME');
  });

  it('posts error when previewing with no recipe', async () => {
    const { host, messages, state } = createHost();
    state.currentRecipe = undefined;

    await handleWebviewMessage(host, {
      type: WEBVIEW_TO_EXTENSION.preview,
      args: {},
      requestId: 3,
    });

    expect(messages).toEqual([
      {
        type: EXTENSION_TO_WEBVIEW.error,
        message: 'No recipe selected',
        requestId: 3,
      },
    ]);
  });

  it('resetSession clears preview in-flight flag', () => {
    const { host, getPreviewInFlight } = createHost();
    host.setPreviewInFlight(true);
    host.resetSession();
    expect(getPreviewInFlight()).toBe(false);
  });
});
