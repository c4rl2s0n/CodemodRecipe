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
        args: { properties: [{ type: 'int', name: 'count' }] },
        requestId: 1,
      })
    ).toBe(true);
    expect(
      isWebviewToExtensionMessage({
        type: WEBVIEW_TO_EXTENSION.pickFile,
        arg: 'path',
        requestId: 2,
      })
    ).toBe(true);
    expect(
      isWebviewToExtensionMessage({
        type: WEBVIEW_TO_EXTENSION.openDiff,
        path: 'lib/a.dart',
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
    expect(
      isWebviewToExtensionMessage({
        type: WEBVIEW_TO_EXTENSION.componentCoverage,
      })
    ).toBe(true);
    expect(
      isWebviewToExtensionMessage({
        type: WEBVIEW_TO_EXTENSION.clearCoverageGaps,
      })
    ).toBe(true);
    expect(
      isWebviewToExtensionMessage({
        type: WEBVIEW_TO_EXTENSION.openCoverageGaps,
        path: 'lib/a.dart',
        gaps: [{ from: 0, to: 4 }],
      })
    ).toBe(true);
  });

  it('rejects openDiff without path and preview without requestId', () => {
    expect(
      isWebviewToExtensionMessage({
        type: WEBVIEW_TO_EXTENSION.openDiff,
      })
    ).toBe(false);
    expect(isWebviewToExtensionMessage({ type: 'not-a-message' })).toBe(false);
    expect(
      isWebviewToExtensionMessage({ type: WEBVIEW_TO_EXTENSION.selectRecipe })
    ).toBe(false);
    expect(
      isWebviewToExtensionMessage({
        type: WEBVIEW_TO_EXTENSION.preview,
        args: { a: 'x' },
      })
    ).toBe(false);
    expect(
      isWebviewToExtensionMessage({
        type: WEBVIEW_TO_EXTENSION.pickFile,
        arg: 'path',
      })
    ).toBe(false);
    expect(
      isWebviewToExtensionMessage({
        type: WEBVIEW_TO_EXTENSION.openCoverageGaps,
        path: 'lib/a.dart',
        gaps: [{ from: 'x', to: 1 }],
      })
    ).toBe(false);
  });
});

describe('isExtensionToWebviewMessage', () => {
  it('accepts filePicked with value or cancelled', () => {
    expect(
      isExtensionToWebviewMessage({
        type: EXTENSION_TO_WEBVIEW.filePicked,
        arg: 'path',
        requestId: 1,
        value: 'lib/x.dart',
      })
    ).toBe(true);
    expect(
      isExtensionToWebviewMessage({
        type: EXTENSION_TO_WEBVIEW.filePicked,
        arg: 'path',
        requestId: 2,
        cancelled: true,
      })
    ).toBe(true);
  });

  it('rejects unknown types', () => {
    expect(isExtensionToWebviewMessage({ type: 'bogus' })).toBe(false);
  });
});
