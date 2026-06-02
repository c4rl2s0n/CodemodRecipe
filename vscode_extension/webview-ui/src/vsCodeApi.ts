import type { ExtensionToWebviewMessage } from './messages';
import { isExtensionToWebviewMessage } from './messages';
import type { PersistedWebviewState } from './types';

const vscode = acquireVsCodeApi();

const extensionListeners = new Set<(msg: ExtensionToWebviewMessage) => void>();
const pendingMessages: ExtensionToWebviewMessage[] = [];

function dispatchExtensionMessage(data: ExtensionToWebviewMessage): void {
  if (extensionListeners.size === 0) {
    pendingMessages.push(data);
    return;
  }
  for (const listener of extensionListeners) {
    listener(data);
  }
}

window.addEventListener('message', (event) => {
  const data = event.data;
  if (!isExtensionToWebviewMessage(data)) {
    return;
  }
  dispatchExtensionMessage(data);
});

export function postToExtension(message: Record<string, unknown>): void {
  vscode.postMessage(message);
}

export function onExtensionMessage(
  handler: (msg: ExtensionToWebviewMessage) => void
): () => void {
  extensionListeners.add(handler);
  if (pendingMessages.length) {
    const queued = [...pendingMessages];
    pendingMessages.length = 0;
    for (const msg of queued) {
      handler(msg);
    }
  }
  return () => extensionListeners.delete(handler);
}

export function getPersistedState(): PersistedWebviewState | undefined {
  const raw = vscode.getState();
  if (!raw || typeof raw !== 'object') {
    return undefined;
  }
  return raw as PersistedWebviewState;
}

export function setPersistedState(state: PersistedWebviewState): void {
  vscode.setState(state);
}
