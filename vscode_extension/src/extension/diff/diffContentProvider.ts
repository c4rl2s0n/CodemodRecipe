import * as vscode from 'vscode';
import { DIFF } from '../constants';

export class DiffContentProvider implements vscode.TextDocumentContentProvider {
  static readonly scheme = DIFF.scheme;

  private readonly contents = new Map<string, string>();
  private readonly emitter = new vscode.EventEmitter<vscode.Uri>();
  readonly onDidChange = this.emitter.event;

  provideTextDocumentContent(uri: vscode.Uri): string {
    return this.contents.get(uri.path) ?? '';
  }

  store(key: string, content: string): vscode.Uri {
    this.contents.set(key, content);
    const uri = vscode.Uri.from({ scheme: DiffContentProvider.scheme, path: key });
    this.emitter.fire(uri);
    return uri;
  }

  clear(): void {
    this.contents.clear();
  }
}
