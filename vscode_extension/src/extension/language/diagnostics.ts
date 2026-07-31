import * as path from 'path';
import * as vscode from 'vscode';
import type { RecipeDiagnostic } from '../../shared';
import { diagnosticRangeParts } from './diagnosticRange';

export class RecipeDiagnostics {
  private readonly collection: vscode.DiagnosticCollection;

  constructor() {
    this.collection = vscode.languages.createDiagnosticCollection('codemodRecipe');
  }

  get disposable(): vscode.Disposable {
    return this.collection;
  }

  clear(): void {
    this.collection.clear();
  }

  publish(diagnostics: readonly RecipeDiagnostic[], workspaceRoot: string): void {
    this.collection.clear();
    const byFile = new Map<string, vscode.Diagnostic[]>();

    for (const item of diagnostics) {
      const severity =
        item.severity === 'error'
          ? vscode.DiagnosticSeverity.Error
          : vscode.DiagnosticSeverity.Warning;
      const sources =
        item.sources && item.sources.length > 0
          ? item.sources
          : [{ file: '', line: undefined, column: undefined }];

      for (const source of sources) {
        const uri = resolveDiagnosticUri(workspaceRoot, source.file);
        const parts = diagnosticRangeParts(source.line, source.column);
        const range = new vscode.Range(
          parts.startLine,
          parts.startCol,
          parts.endLine,
          parts.endCol
        );
        const diagnostic = new vscode.Diagnostic(range, item.message, severity);
        diagnostic.code = item.code;
        diagnostic.source = 'codemod-recipe';
        if (item.hint) {
          diagnostic.relatedInformation = [
            new vscode.DiagnosticRelatedInformation(
              new vscode.Location(uri, range),
              item.hint
            ),
          ];
        }
        const key = uri.toString();
        const list = byFile.get(key) ?? [];
        list.push(diagnostic);
        byFile.set(key, list);
      }
    }

    for (const [uriString, list] of byFile) {
      this.collection.set(vscode.Uri.parse(uriString), list);
    }
  }
}

function resolveDiagnosticUri(workspaceRoot: string, file: string): vscode.Uri {
  if (!file) {
    return vscode.Uri.file(workspaceRoot);
  }
  if (path.isAbsolute(file)) {
    return vscode.Uri.file(file);
  }
  return vscode.Uri.file(path.join(workspaceRoot, file));
}
