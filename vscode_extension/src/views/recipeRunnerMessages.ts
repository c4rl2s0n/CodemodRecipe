import { WEBVIEW_TO_EXTENSION } from '../constants';
import { SelectionPayload } from '../types';

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

export function isWebviewToExtensionMessage(
  value: unknown
): value is WebviewToExtensionMessage {
  return (
    typeof value === 'object' &&
    value !== null &&
    'type' in value &&
    typeof (value as { type: unknown }).type === 'string'
  );
}
