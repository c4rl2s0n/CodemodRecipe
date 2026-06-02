import type { FilePreview, RecipeViewState, SelectionPayload } from './types';

export const WEBVIEW_TO_EXTENSION = {
  showRecipes: 'showRecipes',
  showRunner: 'showRunner',
  selectRecipe: 'selectRecipe',
  refreshRecipes: 'refreshRecipes',
  configureHost: 'configureHost',
  pickFile: 'pickFile',
  pickDirectory: 'pickDirectory',
  preview: 'preview',
  openDiff: 'openDiff',
  apply: 'apply',
} as const;

export const EXTENSION_TO_WEBVIEW = {
  state: 'state',
  filePicked: 'filePicked',
  previewResult: 'previewResult',
  applyResult: 'applyResult',
  error: 'error',
  previewState: 'previewState',
} as const;

export type WebviewToExtensionMessage =
  | { type: typeof WEBVIEW_TO_EXTENSION.showRecipes }
  | { type: typeof WEBVIEW_TO_EXTENSION.showRunner }
  | { type: typeof WEBVIEW_TO_EXTENSION.selectRecipe; id: string }
  | { type: typeof WEBVIEW_TO_EXTENSION.refreshRecipes }
  | { type: typeof WEBVIEW_TO_EXTENSION.configureHost }
  | { type: typeof WEBVIEW_TO_EXTENSION.pickFile; arg: string }
  | { type: typeof WEBVIEW_TO_EXTENSION.pickDirectory; arg: string }
  | {
      type: typeof WEBVIEW_TO_EXTENSION.preview;
      args: Record<string, string>;
      requestId?: number;
    }
  | { type: typeof WEBVIEW_TO_EXTENSION.openDiff; path: string }
  | { type: typeof WEBVIEW_TO_EXTENSION.apply; selection: SelectionPayload };

export type ExtensionToWebviewMessage =
  | { type: typeof EXTENSION_TO_WEBVIEW.state; state: RecipeViewState }
  | { type: typeof EXTENSION_TO_WEBVIEW.filePicked; arg: string; value: string }
  | {
      type: typeof EXTENSION_TO_WEBVIEW.previewResult;
      files: FilePreview[];
      requestId?: number;
      argsKey?: string;
    }
  | { type: typeof EXTENSION_TO_WEBVIEW.applyResult; applied: string[] }
  | {
      type: typeof EXTENSION_TO_WEBVIEW.error;
      message: string;
      requestId?: number;
    }
  | {
      type: typeof EXTENSION_TO_WEBVIEW.previewState;
      inFlight: boolean;
      requestId?: number;
    };

export function isExtensionToWebviewMessage(
  value: unknown
): value is ExtensionToWebviewMessage {
  return (
    typeof value === 'object' &&
    value !== null &&
    'type' in value &&
    typeof (value as { type: unknown }).type === 'string'
  );
}
