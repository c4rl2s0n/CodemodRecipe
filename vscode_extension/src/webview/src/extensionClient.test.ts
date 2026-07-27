import { describe, expect, it, vi } from 'vitest';
import { EXTENSION_TO_WEBVIEW, WEBVIEW_TO_EXTENSION } from '../../shared/messages';
import { createExtensionClient } from './extensionClient';
import type { ExtensionInbound } from './extensionInbound';

describe('createExtensionClient', () => {
  it('posts typed preview messages', () => {
    const post = vi.fn();
    const inbound = { on: vi.fn(), once: vi.fn(), dispose: vi.fn() } as unknown as ExtensionInbound;
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
    const onceHandlers: Array<{
      handler: (msg: { arg: string; value: string }) => void;
      predicate?: (msg: { arg: string }) => boolean;
    }> = [];

    const inbound = {
      on: vi.fn(),
      once: (
        _type: string,
        handler: (msg: { arg: string; value: string }) => void,
        predicate?: (msg: { arg: string }) => boolean
      ) => {
        onceHandlers.push({ handler, predicate });
        return () => {};
      },
      dispose: vi.fn(),
    } as unknown as ExtensionInbound;

    const client = createExtensionClient({ post, inbound });
    const promise = client.pickPath('myArg', true);

    expect(post).toHaveBeenCalledWith({
      type: WEBVIEW_TO_EXTENSION.pickDirectory,
      arg: 'myArg',
    });

    const entry = onceHandlers[0];
    entry.handler({
      arg: 'myArg',
      value: 'lib/foo.dart',
    });

    await expect(promise).resolves.toBe('lib/foo.dart');
  });
});
