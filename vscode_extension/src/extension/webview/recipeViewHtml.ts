import * as fs from 'fs';
import * as vscode from 'vscode';
import { WEBVIEW_ASSETS } from '../constants';

export interface RecipeViewBootConfig {
  autoPreviewDebounceMs: number;
}

export function renderRecipeViewHtml(
  webview: vscode.Webview,
  extensionUri: vscode.Uri,
  boot: RecipeViewBootConfig
): string {
  const nonce = makeNonce();
  const htmlPath = vscode.Uri.joinPath(
    extensionUri,
    ...WEBVIEW_ASSETS.html
  ).fsPath;
  const cssUri = webview.asWebviewUri(
    vscode.Uri.joinPath(extensionUri, ...WEBVIEW_ASSETS.css)
  );
  const scriptUri = webview.asWebviewUri(
    vscode.Uri.joinPath(extensionUri, ...WEBVIEW_ASSETS.script)
  );
  const bootJson = safeJson(boot);
  const template = fs.readFileSync(htmlPath, 'utf8');

  return template
    .replaceAll('{{cspSource}}', webview.cspSource)
    .replaceAll('{{nonce}}', nonce)
    .replaceAll('{{cssUri}}', cssUri.toString())
    .replaceAll('{{scriptUri}}', scriptUri.toString())
    .replaceAll('{{bootJson}}', bootJson);
}

function safeJson(value: unknown): string {
  return JSON.stringify(value).replace(/</g, '\\u003c');
}

function makeNonce(): string {
  const chars =
    'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
  let text = '';
  for (let i = 0; i < 32; i++) {
    text += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return text;
}
