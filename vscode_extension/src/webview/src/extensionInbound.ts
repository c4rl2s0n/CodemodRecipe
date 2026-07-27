import {
  EXTENSION_TO_WEBVIEW,
  type ExtensionToWebviewMessage,
} from './shared';
import { onExtensionMessage } from './vsCodeApi';

type ExtensionMessageType = ExtensionToWebviewMessage['type'];

type Handler<T extends ExtensionMessageType> = (
  msg: Extract<ExtensionToWebviewMessage, { type: T }>
) => void;

type AnyHandler = (msg: ExtensionToWebviewMessage) => void;

export interface ExtensionInbound {
  on<T extends ExtensionMessageType>(type: T, handler: Handler<T>): () => void;
  once<T extends ExtensionMessageType>(
    type: T,
    handler: Handler<T>,
    predicate?: (msg: Extract<ExtensionToWebviewMessage, { type: T }>) => boolean
  ): () => void;
  dispose(): void;
}

export function createExtensionInbound(): ExtensionInbound {
  const listeners = new Map<ExtensionMessageType, Set<AnyHandler>>();
  let unsubscribeTransport: (() => void) | undefined;

  function addListener<T extends ExtensionMessageType>(
    type: T,
    handler: Handler<T>,
    once: boolean,
    predicate?: (msg: Extract<ExtensionToWebviewMessage, { type: T }>) => boolean
  ): () => void {
    const wrapped: AnyHandler = (msg) => {
      if (msg.type !== type) {
        return;
      }
      const typed = msg as Extract<ExtensionToWebviewMessage, { type: T }>;
      if (predicate && !predicate(typed)) {
        return;
      }
      if (once) {
        remove();
      }
      handler(typed);
    };

    const remove = () => {
      const set = listeners.get(type);
      if (set) {
        set.delete(wrapped);
        if (set.size === 0) {
          listeners.delete(type);
        }
      }
    };

    let set = listeners.get(type);
    if (!set) {
      set = new Set();
      listeners.set(type, set);
    }
    set.add(wrapped);
    return remove;
  }

  function dispatch(msg: ExtensionToWebviewMessage): void {
    const set = listeners.get(msg.type);
    if (!set) {
      return;
    }
    for (const handler of [...set]) {
      handler(msg);
    }
  }

  function ensureTransport(): void {
    if (unsubscribeTransport) {
      return;
    }
    unsubscribeTransport = onExtensionMessage(dispatch);
  }

  return {
    on<T extends ExtensionMessageType>(type: T, handler: Handler<T>): () => void {
      ensureTransport();
      return addListener(type, handler, false);
    },
    once<T extends ExtensionMessageType>(
      type: T,
      handler: Handler<T>,
      predicate?: (msg: Extract<ExtensionToWebviewMessage, { type: T }>) => boolean
    ): () => void {
      ensureTransport();
      return addListener(type, handler, true, predicate);
    },
    dispose(): void {
      unsubscribeTransport?.();
      unsubscribeTransport = undefined;
      listeners.clear();
    },
  };
}