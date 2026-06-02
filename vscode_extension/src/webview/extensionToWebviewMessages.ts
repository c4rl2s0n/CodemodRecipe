import { EXTENSION_TO_WEBVIEW } from '../constants';
import { FilePreview } from '../types';
import { RecipeViewState } from './webviewState';

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
