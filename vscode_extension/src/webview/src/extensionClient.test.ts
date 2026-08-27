import { describe, expect, it, vi } from 'vitest';
import {
  EXTENSION_TO_WEBVIEW,
  WEBVIEW_TO_EXTENSION,
  type ExtensionToWebviewMessage,
} from '../../shared/messages';
import {
  createExtensionClient,
  createRequestRpc,
} from './extensionClient';
import type { ExtensionInbound } from './extensionInbound';

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

  return { inbound, emit, handlers };
}

describe('createExtensionClient', () => {
  it('posts typed preview messages', () => {
    const post = vi.fn();
    const { inbound } = mockInbound();
    const client = createExtensionClient({ post, inbound });

    client.requestPreview({ foo: 'bar' }, 3);

    expect(post).toHaveBeenCalledWith({
      type: WEBVIEW_TO_EXTENSION.preview,
      args: { foo: 'bar' },
      requestId: 3,
    });
  });

  it('pickPath resolves when matching filePicked arrives', async () => {
    const post = vi.fn();
    const { inbound, emit } = mockInbound();
    const client = createExtensionClient({ post, inbound });
    const promise = client.pickPath('myArg', true);

    expect(post).toHaveBeenCalledWith({
      type: WEBVIEW_TO_EXTENSION.pickDirectory,
      arg: 'myArg',
      requestId: 1,
    });

    const requestId = post.mock.calls[0][0].requestId as number;
    emit({
      type: EXTENSION_TO_WEBVIEW.filePicked,
      arg: 'myArg',
      requestId,
      value: 'lib/foo.dart',
    });

    await expect(promise).resolves.toBe('lib/foo.dart');
  });

  it('pickPath resolves undefined on cancel', async () => {
    const post = vi.fn();
    const { inbound, emit } = mockInbound();
    const client = createExtensionClient({ post, inbound });
    const promise = client.pickPath('path', false);

    const requestId = post.mock.calls[0][0].requestId as number;
    emit({
      type: EXTENSION_TO_WEBVIEW.filePicked,
      arg: 'path',
      requestId,
      cancelled: true,
    });

    await expect(promise).resolves.toBeUndefined();
  });

  it('second pickPath cancels the first waiter', async () => {
    const post = vi.fn();
    const { inbound, emit } = mockInbound();
    const client = createExtensionClient({ post, inbound });

    const first = client.pickPath('path', false);
    const second = client.pickPath('path', false);
    const secondId = post.mock.calls[1][0].requestId as number;

    await expect(first).resolves.toBeUndefined();

    emit({
      type: EXTENSION_TO_WEBVIEW.filePicked,
      arg: 'path',
      requestId: secondId,
      value: 'new.dart',
    });

    await expect(second).resolves.toBe('new.dart');
  });
});

describe('createRequestRpc', () => {
  it('times out with an error result', async () => {
    const post = vi.fn();
    const { inbound } = mockInbound();
    const rpc = createRequestRpc({ post, inbound });

    const { promise } = rpc.request<string>({
      terminalTypes: [EXTENSION_TO_WEBVIEW.filePicked],
      timeoutMs: 20,
      send: () => {},
      match: () => undefined,
    });

    await expect(promise).resolves.toEqual({
      status: 'error',
      message: 'Request timed out',
    });
  });

  it('matches error terminals by requestId', async () => {
    const post = vi.fn();
    const { inbound, emit } = mockInbound();
    const rpc = createRequestRpc({ post, inbound });

    const { promise } = rpc.request<string>({
      terminalTypes: [
        EXTENSION_TO_WEBVIEW.filePicked,
        EXTENSION_TO_WEBVIEW.error,
      ],
      send: (requestId) => {
        post({ type: WEBVIEW_TO_EXTENSION.pickFile, arg: 'a', requestId });
      },
      match: (msg, requestId) => {
        if (msg.type === EXTENSION_TO_WEBVIEW.error && msg.requestId === requestId) {
          return { status: 'error', message: msg.message };
        }
        return undefined;
      },
    });

    const requestId = post.mock.calls[0][0].requestId as number;
    emit({
      type: EXTENSION_TO_WEBVIEW.error,
      message: 'boom',
      requestId,
    });

    await expect(promise).resolves.toEqual({
      status: 'error',
      message: 'boom',
    });
  });

  it('cancel settles as cancelled and unsubscribes', async () => {
    const post = vi.fn();
    const { inbound, handlers } = mockInbound();
    const rpc = createRequestRpc({ post, inbound });

    const { promise, cancel } = rpc.request<string>({
      terminalTypes: [EXTENSION_TO_WEBVIEW.filePicked],
      send: () => {},
      match: () => undefined,
    });

    cancel();
    await expect(promise).resolves.toEqual({ status: 'cancelled' });
    expect(handlers.get(EXTENSION_TO_WEBVIEW.filePicked)?.size ?? 0).toBe(0);
  });
});
