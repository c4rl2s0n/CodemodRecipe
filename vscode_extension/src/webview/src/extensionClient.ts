import {
  EXTENSION_TO_WEBVIEW,
  WEBVIEW_TO_EXTENSION,
  type ExtensionToWebviewMessage,
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
  pickPath(arg: string, directory: boolean): Promise<string>;
  requestPreview(args: Record<string, string>, requestId: number): void;
  openDiff(path: string, patchIndex: number): void;
  apply(selection: SelectionPayload): void;
  invokeRecipe(
    recipeId: string,
    mode: 'auto' | 'run' | 'open',
    args?: Record<string, string>
  ): void;
  createShortcut(recipeId: string, args?: Record<string, string>): void;
}

export function createExtensionClient(deps: ExtensionClientDeps): ExtensionClient {
  const { post, inbound } = deps;

  function pickPath(arg: string, directory: boolean): Promise<string> {
    return new Promise((resolve) => {
      inbound.once(
        EXTENSION_TO_WEBVIEW.filePicked,
        (msg) => {
          resolve(msg.value);
        },
        (msg) => msg.arg === arg
      );
      post({
        type: directory
          ? WEBVIEW_TO_EXTENSION.pickDirectory
          : WEBVIEW_TO_EXTENSION.pickFile,
        arg,
      });
    });
  }

  return {
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
      post({ type: WEBVIEW_TO_EXTENSION.pickFile, arg });
    },
    pickDirectory(arg: string) {
      post({ type: WEBVIEW_TO_EXTENSION.pickDirectory, arg });
    },
    pickPath,
    requestPreview(args: Record<string, string>, requestId: number) {
      post({ type: WEBVIEW_TO_EXTENSION.preview, args, requestId });
    },
    openDiff(path: string, patchIndex: number) {
      post({ type: WEBVIEW_TO_EXTENSION.openDiff, path, patchIndex });
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
  };
}
