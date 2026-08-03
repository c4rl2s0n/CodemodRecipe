import { describe, expect, it } from 'vitest';
import {
  EXTENSION_TO_WEBVIEW,
  WEBVIEW_TO_EXTENSION,
  isExtensionToWebviewMessage,
  isWebviewToExtensionMessage,
} from './messages';

describe('isWebviewToExtensionMessage', () => {
  it('accepts valid messages', () => {
    expect(isWebviewToExtensionMessage({ type: WEBVIEW_TO_EXTENSION.ready })).toBe(
      true
    );
    expect(
      isWebviewToExtensionMessage({
        type: WEBVIEW_TO_EXTENSION.selectRecipe,
        id: 'r1',
      })
    ).toBe(true);
    expect(
      isWebviewToExtensionMessage({
        type: WEBVIEW_TO_EXTENSION.preview,
        args: { a: 'b' },
        requestId: 1,
      })
    ).toBe(true);
    expect(
      isWebviewToExtensionMessage({
        type: WEBVIEW_TO_EXTENSION.openDiff,
        path: 'lib/a.dart',
        patchIndex: -1,
      })
    ).toBe(true);
    expect(
      isWebviewToExtensionMessage({
        type: WEBVIEW_TO_EXTENSION.invokeRecipe,
        recipeId: 'r1',
        mode: 'auto',
        args: { file: 'a.dart' },
      })
    ).toBe(true);
    expect(
      isWebviewToExtensionMessage({
        type: WEBVIEW_TO_EXTENSION.createShortcut,
        recipeId: 'r1',
      })
    ).toBe(true);
  });

  it('rejects openDiff without patchIndex', () => {
    expect(
      isWebviewToExtensionMessage({
        type: WEBVIEW_TO_EXTENSION.openDiff,
        path: 'lib/a.dart',
      })
    ).toBe(false);
    expect(isWebviewToExtensionMessage({ type: 'not-a-message' })).toBe(false);
    expect(
      isWebviewToExtensionMessage({ type: WEBVIEW_TO_EXTENSION.selectRecipe })
    ).toBe(false);
    expect(
      isWebviewToExtensionMessage({
        type: WEBVIEW_TO_EXTENSION.preview,
        args: { a: 1 },
      })
    ).toBe(false);
  });
});

describe('isExtensionToWebviewMessage', () => {
  it('accepts filePicked', () => {
    expect(
      isExtensionToWebviewMessage({
        type: EXTENSION_TO_WEBVIEW.filePicked,
        arg: 'path',
        value: 'lib/x.dart',
      })
    ).toBe(true);
  });

  it('rejects unknown types', () => {
    expect(isExtensionToWebviewMessage({ type: 'bogus' })).toBe(false);
  });
});
