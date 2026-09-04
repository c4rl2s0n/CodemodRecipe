import {
  EXTENSION_TO_WEBVIEW,
  WEBVIEW_TO_EXTENSION,
  type ArgValues,
  type ExtensionToWebviewMessage,
  type RpcResult,
  type SelectionPayload,
  type WebviewToExtensionMessage,
} from './shared';
import type { ExtensionInbound } from './extensionInbound';

export interface ExtensionClientDeps {
  post: (message: WebviewToExtensionMessage) => void;
  inbound: ExtensionInbound;
}

export interface ExtensionClient {
  notifyReady(): void;
  retryBootstrap(): void;
  selectRecipe(id: string): void;
  refreshRecipes(): void;
  configureHost(): void;
  scaffoldProject(): void;
  openRecipeFile(id: string): void;
  pickFile(arg: string): void;
  pickDirectory(arg: string): void;
  /** Resolves to the selected path, or `undefined` when the dialog was cancelled. */
  pickPath(arg: string, directory: boolean): Promise<string | undefined>;
  requestPreview(args: ArgValues, requestId: number): void;
  openDiff(path: string): void;
  apply(selection: SelectionPayload): void;
  invokeRecipe(
    recipeId: string,
    mode: 'auto' | 'run' | 'open',
    args?: ArgValues
  ): void;
  createShortcut(recipeId: string, args?: ArgValues): void;
  selectComponent(componentId: string): void;
  selectComponentInstance(instanceId: string): void;
  selectComponentAction(action: string): void;
  componentScan(): void;
  componentPreview(args: ArgValues, requestId: number): void;
  componentApply(args: ArgValues, requestId: number): void;
  /** Allocate a monotonic request id for fire-and-forget correlated ops (preview). */
  nextRequestId(): number;
}

type ExtensionMessageType = ExtensionToWebviewMessage['type'];

export interface RequestOptions<T> {
  send: (requestId: number) => void;
  /**
   * For each inbound message, return an RpcResult when this is the terminal
   * reply for `requestId`; otherwise return undefined to keep waiting.
   */
  match: (
    msg: ExtensionToWebviewMessage,
    requestId: number
  ) => RpcResult<T> | undefined;
  /** Message types to subscribe to while waiting. */
  terminalTypes: readonly ExtensionMessageType[];
  timeoutMs?: number;
}

/**
 * ACK-required RPC: posts a correlated request and waits for exactly one
 * terminal ok | error | cancelled matched by requestId.
 */
export function createRequestRpc(deps: ExtensionClientDeps) {
  let nextId = 1;

  function nextRequestId(): number {
    return nextId++;
  }

  function request<T>(options: RequestOptions<T>): {
    promise: Promise<RpcResult<T>>;
    cancel: () => void;
  } {
    const requestId = nextRequestId();
    let settled = false;
    const unsubs: Array<() => void> = [];
    let timer: ReturnType<typeof setTimeout> | undefined;
    let settle!: (result: RpcResult<T>) => void;

    const promise = new Promise<RpcResult<T>>((resolve) => {
      settle = (result: RpcResult<T>) => {
        if (settled) {
          return;
        }
        settled = true;
        if (timer !== undefined) {
          clearTimeout(timer);
        }
        for (const off of unsubs) {
          off();
        }
        resolve(result);
      };

      for (const type of options.terminalTypes) {
        unsubs.push(
          deps.inbound.on(type, (msg) => {
            const result = options.match(msg, requestId);
            if (result) {
              settle(result);
            }
          })
        );
      }

      if (options.timeoutMs !== undefined && options.timeoutMs > 0) {
        timer = setTimeout(() => {
          settle({ status: 'error', message: 'Request timed out' });
        }, options.timeoutMs);
      }

      options.send(requestId);
    });

    return {
      promise,
      cancel: () => settle({ status: 'cancelled' }),
    };
  }

  return { nextRequestId, request };
}

export function createExtensionClient(deps: ExtensionClientDeps): ExtensionClient {
  const { post } = deps;
  const rpc = createRequestRpc(deps);

  // Cancel an in-flight pick for the same arg when a new pick starts.
  const pickCancelByArg = new Map<string, () => void>();

  async function pickPath(
    arg: string,
    directory: boolean
  ): Promise<string | undefined> {
    pickCancelByArg.get(arg)?.();

    const { promise, cancel } = rpc.request<string>({
      terminalTypes: [
        EXTENSION_TO_WEBVIEW.filePicked,
        EXTENSION_TO_WEBVIEW.error,
      ],
      send: (requestId) => {
        post({
          type: directory
            ? WEBVIEW_TO_EXTENSION.pickDirectory
            : WEBVIEW_TO_EXTENSION.pickFile,
          arg,
          requestId,
        });
      },
      match: (msg, requestId) => {
        if (msg.type === EXTENSION_TO_WEBVIEW.error) {
          if (msg.requestId !== requestId) {
            return undefined;
          }
          return { status: 'error', message: msg.message };
        }
        if (msg.type !== EXTENSION_TO_WEBVIEW.filePicked) {
          return undefined;
        }
        if (msg.requestId !== requestId || msg.arg !== arg) {
          return undefined;
        }
        if (msg.cancelled) {
          return { status: 'cancelled' };
        }
        if (typeof msg.value === 'string') {
          return { status: 'ok', value: msg.value };
        }
        return { status: 'cancelled' };
      },
    });

    pickCancelByArg.set(arg, () => {
      cancel();
      pickCancelByArg.delete(arg);
    });

    const result = await promise;
    if (pickCancelByArg.get(arg)) {
      pickCancelByArg.delete(arg);
    }

    if (result.status === 'ok') {
      return result.value;
    }
    if (result.status === 'cancelled') {
      return undefined;
    }
    throw new Error(result.message);
  }

  return {
    nextRequestId: rpc.nextRequestId,
    notifyReady() {
      post({ type: WEBVIEW_TO_EXTENSION.ready });
    },
    retryBootstrap() {
      post({ type: WEBVIEW_TO_EXTENSION.bootstrapRetry });
    },
    selectRecipe(id: string) {
      post({ type: WEBVIEW_TO_EXTENSION.selectRecipe, id });
    },
    refreshRecipes() {
      post({ type: WEBVIEW_TO_EXTENSION.refreshRecipes });
    },
    configureHost() {
      post({ type: WEBVIEW_TO_EXTENSION.configureHost });
    },
    scaffoldProject() {
      post({ type: WEBVIEW_TO_EXTENSION.scaffoldProject });
    },
    openRecipeFile(id: string) {
      post({ type: WEBVIEW_TO_EXTENSION.openRecipeFile, id });
    },
    pickFile(arg: string) {
      void pickPath(arg, false);
    },
    pickDirectory(arg: string) {
      void pickPath(arg, true);
    },
    pickPath,
    requestPreview(args: ArgValues, requestId: number) {
      post({ type: WEBVIEW_TO_EXTENSION.preview, args, requestId });
    },
    openDiff(path: string) {
      post({ type: WEBVIEW_TO_EXTENSION.openDiff, path });
    },
    apply(selection: SelectionPayload) {
      post({ type: WEBVIEW_TO_EXTENSION.apply, selection });
    },
    invokeRecipe(recipeId, mode, args) {
      post({
        type: WEBVIEW_TO_EXTENSION.invokeRecipe,
        recipeId,
        mode,
        ...(args ? { args } : {}),
      });
    },
    createShortcut(recipeId, args) {
      post({
        type: WEBVIEW_TO_EXTENSION.createShortcut,
        recipeId,
        ...(args ? { args } : {}),
      });
    },
    selectComponent(componentId: string) {
      post({ type: WEBVIEW_TO_EXTENSION.selectComponent, componentId });
    },
    selectComponentInstance(instanceId: string) {
      post({ type: WEBVIEW_TO_EXTENSION.selectComponentInstance, instanceId });
    },
    selectComponentAction(action: string) {
      post({ type: WEBVIEW_TO_EXTENSION.selectComponentAction, action });
    },
    componentScan() {
      post({ type: WEBVIEW_TO_EXTENSION.componentScan });
    },
    componentPreview(args: ArgValues, requestId: number) {
      post({ type: WEBVIEW_TO_EXTENSION.componentPreview, args, requestId });
    },
    componentApply(args: ArgValues, requestId: number) {
      post({ type: WEBVIEW_TO_EXTENSION.componentApply, args, requestId });
    },
  };
}
