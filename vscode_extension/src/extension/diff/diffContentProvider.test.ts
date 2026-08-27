import { describe, expect, it, vi } from 'vitest';
import {
  DiffContentProvider,
  diffModifiedKey,
  diffOriginalKey,
  diffSafePath,
} from './diffContentProvider';

vi.mock('vscode', () => {
  return {
    Uri: {
      from: (parts: { scheme: string; path: string }) => ({
        scheme: parts.scheme,
        path: parts.path,
        toString: () => `${parts.scheme}:${parts.path}`,
      }),
    },
    EventEmitter: class {
      private listeners: Array<(e: unknown) => void> = [];
      event = (listener: (e: unknown) => void) => {
        this.listeners.push(listener);
        return { dispose: () => undefined };
      };
      fire(e: unknown) {
        for (const listener of this.listeners) {
          listener(e);
        }
      }
    },
  };
});

describe('DiffContentProvider', () => {
  it('has / keys track stored content', () => {
    const provider = new DiffContentProvider();
    const key = diffModifiedKey('lib/a.dart');
    expect(provider.has(key)).toBe(false);
    provider.store(key, 'class A {}\n');
    expect(provider.has(key)).toBe(true);
    expect(provider.keys()).toContain(key);
    expect(
      provider.provideTextDocumentContent({ path: key } as never)
    ).toBe('class A {}\n');
  });

  it('store overwrite fires onDidChange with updated content', () => {
    const provider = new DiffContentProvider();
    const key = diffOriginalKey('lib/foo.dart');
    const seen: string[] = [];
    provider.onDidChange((uri) => {
      seen.push((uri as { path: string }).path);
    });
    provider.store(key, 'old');
    provider.store(key, 'new');
    expect(seen).toEqual([key, key]);
    expect(
      provider.provideTextDocumentContent({ path: key } as never)
    ).toBe('new');
  });

  it('diffSafePath sanitizes path segments', () => {
    expect(diffSafePath('lib/foo-bar.dart')).toBe('lib_foo_bar_dart');
  });
});
