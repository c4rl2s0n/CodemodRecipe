import { describe, expect, it, vi } from 'vitest';
import { ref } from 'vue';
import {
  EXTENSION_TO_WEBVIEW,
  RUNNER_SESSION_PHASE,
  RUNNER_TABS,
  type RecipeSchema,
  type RecipeViewState,
  type ExtensionToWebviewMessage,
} from '../shared';
import type { ExtensionClient } from '../extensionClient';
import type { ExtensionInbound } from '../extensionInbound';

vi.mock('../vsCodeApi', () => ({
  getPersistedState: () => undefined,
  setPersistedState: vi.fn(),
}));

import { useRunnerController } from './useRunnerController';

function mockInbound() {
  const handlers = new Map<
    string,
    Set<(msg: ExtensionToWebviewMessage) => void>
  >();

  const inbound = {
    on: (type: string, handler: (msg: ExtensionToWebviewMessage) => void) => {
      let set = handlers.get(type);
      if (!set) {
        set = new Set();
        handlers.set(type, set);
      }
      set.add(handler);
      return () => set!.delete(handler);
    },
    once: vi.fn(),
    dispose: vi.fn(),
  } as unknown as ExtensionInbound;

  function emit(msg: ExtensionToWebviewMessage) {
    const set = handlers.get(msg.type);
    if (!set) return;
    for (const handler of [...set]) {
      handler(msg);
    }
  }

  return { inbound, emit };
}

const recipe: RecipeSchema = {
  id: 'r1',
  name: 'R1',
  description: 'test',
  args: [
    {
      name: 'a',
      abbr: null,
      help: null,
      required: false,
      defaultsTo: null,
      inputKind: 'text',
      options: [],
      allowCustomValue: true,
      contextKey: null,
    },
    {
      name: 'b',
      abbr: null,
      help: null,
      required: false,
      defaultsTo: null,
      inputKind: 'text',
      options: [],
      allowCustomValue: true,
      contextKey: null,
    },
  ],
};

function baseState(overrides: Partial<RecipeViewState> = {}): RecipeViewState {
  return {
    recipes: [recipe],
    diagnostics: [],
    recipesRefreshing: false,
    bootstrapInFlight: false,
    bootstrapPhase: 'ready',
    recipe,
    initialArgs: {},
    activeTab: RUNNER_TABS.runner,
    contextMatches: [],
    slotsByRecipe: {},
    session: { id: 1, phase: RUNNER_SESSION_PHASE.idle },
    ...overrides,
  };
}

describe('useRunnerController', () => {
  it('sends a new preview after debounce while one is in flight (supersede)', async () => {
    vi.useFakeTimers();
    const { inbound } = mockInbound();
    let requestCounter = 0;
    const requestPreview = vi.fn();
    const client = {
      nextRequestId: () => ++requestCounter,
      requestPreview,
    } as unknown as ExtensionClient;

    const recipeRef = ref(recipe);
    const activeTab = ref(RUNNER_TABS.runner);
    const ctrl = useRunnerController({
      client,
      inbound,
      recipe: recipeRef,
      autoPreviewDebounceMs: { value: 10 },
      activeTab,
      setActiveTab: (tab) => {
        activeTab.value = tab as typeof RUNNER_TABS.runner;
      },
    });

    ctrl.argValues.value = { a: '1', b: '2' };
    ctrl.onArgsChanged(true);
    expect(requestPreview).toHaveBeenCalledTimes(1);

    // Change args while in flight — debounce then send again (host supersedes).
    ctrl.argValues.value = { a: '1', b: '3' };
    ctrl.onArgsChanged(false);
    expect(ctrl.previewStatus.value).toBe('Preview out of date');
    expect(requestPreview).toHaveBeenCalledTimes(1);

    vi.advanceTimersByTime(10);
    expect(requestPreview).toHaveBeenCalledTimes(2);
    expect(requestPreview.mock.calls[1][0]).toEqual({ a: '1', b: '3' });

    vi.useRealTimers();
  });

  it('skips preview while required args are missing', () => {
    vi.useFakeTimers();
    const { inbound } = mockInbound();
    const requestPreview = vi.fn();
    const client = {
      nextRequestId: () => 1,
      requestPreview,
    } as unknown as ExtensionClient;

    const requiredRecipe: RecipeSchema = {
      ...recipe,
      args: [
        {
          name: 'path',
          abbr: null,
          help: null,
          required: true,
          defaultsTo: null,
          inputKind: 'file',
          options: [],
          allowCustomValue: true,
          contextKey: null,
        },
      ],
    };
    const recipeRef = ref(requiredRecipe);
    const activeTab = ref(RUNNER_TABS.runner);
    const ctrl = useRunnerController({
      client,
      inbound,
      recipe: recipeRef,
      autoPreviewDebounceMs: { value: 10 },
      activeTab,
      setActiveTab: () => {},
    });

    ctrl.argValues.value = { path: '' };
    ctrl.onArgsChanged(true);
    expect(requestPreview).not.toHaveBeenCalled();
    expect(ctrl.previewStatus.value).toContain('Missing required');

    vi.useRealTimers();
  });

  it('does not markHandled on busy previewState ACK', () => {
    const { inbound, emit } = mockInbound();
    let requestCounter = 0;
    const requestPreview = vi.fn();
    const client = {
      nextRequestId: () => ++requestCounter,
      requestPreview,
    } as unknown as ExtensionClient;

    const recipeRef = ref(recipe);
    const activeTab = ref(RUNNER_TABS.runner);
    const ctrl = useRunnerController({
      client,
      inbound,
      recipe: recipeRef,
      autoPreviewDebounceMs: { value: 10 },
      activeTab,
      setActiveTab: () => {},
    });

    ctrl.argValues.value = { a: '1', b: '2' };
    ctrl.onArgsChanged(true);
    const id = requestPreview.mock.calls[0][1] as number;

    emit({
      type: EXTENSION_TO_WEBVIEW.previewState,
      inFlight: true,
      requestId: id,
    });
    expect(ctrl.previewInFlight.value).toBe(true);

    // A subsequent result for that id must still be accepted (not stale).
    emit({
      type: EXTENSION_TO_WEBVIEW.previewResult,
      files: [{ path: 'x.dart', kind: 'edit', isNew: false, skipped: false }],
      requestId: id,
      argsKey: '{"a":"1","b":"2"}',
    });
    expect(ctrl.files.value).toHaveLength(1);
    expect(ctrl.previewInFlight.value).toBe(false);
  });

  it('stores runtime diagnostics from error messages', () => {
    const { inbound, emit } = mockInbound();
    const client = {
      nextRequestId: () => 1,
      requestPreview: vi.fn(),
    } as unknown as ExtensionClient;

    const recipeRef = ref(recipe);
    const activeTab = ref(RUNNER_TABS.runner);
    const ctrl = useRunnerController({
      client,
      inbound,
      recipe: recipeRef,
      autoPreviewDebounceMs: { value: 50 },
      activeTab,
      setActiveTab: () => {},
    });

    emit({
      type: EXTENSION_TO_WEBVIEW.error,
      message: 'Lua boom',
      requestId: 1,
      diagnostics: [
        {
          severity: 'error',
          code: 'E_RECIPE_RUNTIME',
          message: 'Lua boom',
          hint: 'Fix apply()',
        },
      ],
    });

    expect(ctrl.errorMessage.value).toBe('Lua boom');
    expect(ctrl.runtimeDiagnostics.value[0]?.code).toBe('E_RECIPE_RUNTIME');
  });

  it('clears in-flight on recipe change so a new preview can run', async () => {
    vi.useFakeTimers();
    const { inbound, emit } = mockInbound();
    let requestCounter = 0;
    const requestPreview = vi.fn();
    const client = {
      nextRequestId: () => ++requestCounter,
      requestPreview,
    } as unknown as ExtensionClient;

    const recipeRef = ref(recipe);
    const activeTab = ref(RUNNER_TABS.runner);
    const ctrl = useRunnerController({
      client,
      inbound,
      recipe: recipeRef,
      autoPreviewDebounceMs: { value: 10 },
      activeTab,
      setActiveTab: (tab) => {
        activeTab.value = tab as typeof RUNNER_TABS.runner;
      },
    });

    emit({ type: EXTENSION_TO_WEBVIEW.state, state: baseState() });
    vi.advanceTimersByTime(10);
    expect(requestPreview).toHaveBeenCalled();
    const callsBefore = requestPreview.mock.calls.length;

    // Simulate stuck local in-flight then recipe switch via state
    ctrl.previewInFlight.value = true;
    const recipe2 = { ...recipe, id: 'r2', name: 'R2' };
    recipeRef.value = recipe2;
    emit({
      type: EXTENSION_TO_WEBVIEW.state,
      state: baseState({
        recipe: recipe2,
        recipes: [recipe2],
        session: { id: 1, phase: RUNNER_SESSION_PHASE.idle },
      }),
    });

    expect(ctrl.previewInFlight.value).toBe(false);
    vi.advanceTimersByTime(10);
    expect(requestPreview.mock.calls.length).toBeGreaterThan(callsBefore);

    vi.useRealTimers();
  });

  it('does not mark preview out of date on same-recipe context postState', () => {
    vi.useFakeTimers();
    const { inbound, emit } = mockInbound();
    const requestPreview = vi.fn();
    const client = {
      nextRequestId: () => 1,
      requestPreview,
    } as unknown as ExtensionClient;

    const recipeRef = ref(recipe);
    const activeTab = ref(RUNNER_TABS.runner);
    const ctrl = useRunnerController({
      client,
      inbound,
      recipe: recipeRef,
      autoPreviewDebounceMs: { value: 50 },
      activeTab,
      setActiveTab: (tab) => {
        activeTab.value = tab as typeof RUNNER_TABS.runner;
      },
    });

    emit({ type: EXTENSION_TO_WEBVIEW.state, state: baseState() });
    vi.advanceTimersByTime(50);
    requestPreview.mockClear();
    ctrl.previewStatus.value = '';

    emit({
      type: EXTENSION_TO_WEBVIEW.state,
      state: baseState({
        contextMatches: [
          {
            recipeId: 'r1',
            name: 'R1',
            description: '',
            args: {},
            complete: true,
          },
        ],
      }),
    });

    expect(ctrl.previewStatus.value).not.toBe('Preview out of date');
    expect(requestPreview).not.toHaveBeenCalled();
    vi.useRealTimers();
  });

  it('keeps Recipes tab across subsequent postState', () => {
    const { inbound, emit } = mockInbound();
    const client = {
      nextRequestId: () => 1,
      requestPreview: vi.fn(),
    } as unknown as ExtensionClient;

    const recipeRef = ref(recipe);
    const activeTab = ref<typeof RUNNER_TABS.recipes | typeof RUNNER_TABS.runner>(
      RUNNER_TABS.runner
    );
    useRunnerController({
      client,
      inbound,
      recipe: recipeRef,
      autoPreviewDebounceMs: { value: 50 },
      activeTab,
      setActiveTab: (tab) => {
        activeTab.value = tab as typeof RUNNER_TABS.recipes;
      },
    });

    emit({ type: EXTENSION_TO_WEBVIEW.state, state: baseState() });
    activeTab.value = RUNNER_TABS.recipes;

    emit({
      type: EXTENSION_TO_WEBVIEW.state,
      state: baseState({
        activeTab: RUNNER_TABS.runner,
        contextMatches: [],
      }),
    });

    expect(activeTab.value).toBe(RUNNER_TABS.recipes);
  });

  it('canApply is true when host argsKey matches canonicalize order', () => {
    const { inbound, emit } = mockInbound();
    const client = {
      nextRequestId: () => 1,
      requestPreview: vi.fn(),
    } as unknown as ExtensionClient;

    const recipeRef = ref(recipe);
    const activeTab = ref(RUNNER_TABS.runner);
    const ctrl = useRunnerController({
      client,
      inbound,
      recipe: recipeRef,
      autoPreviewDebounceMs: { value: 50 },
      activeTab,
      setActiveTab: () => {},
    });

    ctrl.argValues.value = { b: '2', a: '1' };
    emit({
      type: EXTENSION_TO_WEBVIEW.previewResult,
      files: [{ path: 'x.dart', kind: 'edit', isNew: false, skipped: false }],
      requestId: 1,
      // Sorted key order from shared argsKey
      argsKey: '{"a":"1","b":"2"}',
    });

    expect(ctrl.canApply.value).toBe(true);
  });
});
