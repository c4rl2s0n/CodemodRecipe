import * as fs from 'fs';
import * as path from 'path';
import * as vscode from 'vscode';
import type { HostBridge } from '../../host/hostBridge';
import type { RecipeRepository } from '../../recipes/recipeRepository';
import type { RecipeSchema } from '../../../shared';
import {
  collectCompletionSuggestions,
  type CompletionKind,
  type CompletionSuggestion,
} from '../completionLogic';
import { hostMethodByName } from '../hostApiSurface';
import { findNearbyRunRecipeId, inRunArgsTable } from '../luauContext';

const KIND_MAP: Record<CompletionKind, vscode.CompletionItemKind> = {
  method: vscode.CompletionItemKind.Method,
  field: vscode.CompletionItemKind.Field,
  property: vscode.CompletionItemKind.Property,
  module: vscode.CompletionItemKind.Module,
  reference: vscode.CompletionItemKind.Reference,
  value: vscode.CompletionItemKind.Value,
  enumMember: vscode.CompletionItemKind.EnumMember,
  variable: vscode.CompletionItemKind.Variable,
  enum: vscode.CompletionItemKind.Enum,
};

function toVscodeItem(
  suggestion: CompletionSuggestion,
  line: number
): vscode.CompletionItem {
  const item = new vscode.CompletionItem(
    suggestion.label,
    KIND_MAP[suggestion.kind]
  );
  if (suggestion.detail !== undefined) {
    item.detail = suggestion.detail;
  }
  if (suggestion.insertText !== undefined) {
    item.insertText = suggestion.insertText;
  }
  if (suggestion.sortText !== undefined) {
    item.sortText = suggestion.sortText;
  }
  if (suggestion.range) {
    item.range = new vscode.Range(
      line,
      suggestion.range.start,
      line,
      suggestion.range.end
    );
  }
  return item;
}

export class RecipeCompletionProvider implements vscode.CompletionItemProvider {
  constructor(
    private readonly repository: RecipeRepository,
    private readonly bridge: HostBridge,
    private readonly isUnderCodemod: (uri: vscode.Uri) => boolean
  ) {}

  async provideCompletionItems(
    document: vscode.TextDocument,
    position: vscode.Position
  ): Promise<vscode.CompletionItem[] | undefined> {
    if (!this.isUnderCodemod(document.uri)) {
      return;
    }

    const line = document.lineAt(position.line).text;
    const before = line.slice(0, position.character);
    const docDir = path.dirname(document.uri.fsPath);
    const workspaceFolder = vscode.workspace.getWorkspaceFolder(document.uri);
    const workspaceRoot = workspaceFolder?.uri.fsPath;

    const describeCache = new Map<string, RecipeSchema | undefined>();
    if (inRunArgsTable(document, position.line, position.character)) {
      const childId = findNearbyRunRecipeId(document, position.line);
      if (childId) {
        try {
          await this.bridge.ensureHost();
        } catch {
          // ignore
        }
        describeCache.set(
          childId,
          await this.repository.describeCached(childId)
        );
      }
    }

    const suggestions = collectCompletionSuggestions({
      line,
      before,
      lineNumber: position.line,
      character: position.character,
      document,
      catalog: {
        recipeIds: this.repository.getRecipes().map((r) => r.id),
        mapIds: this.repository.getMapIds(),
        varIds: this.repository.getVarIds(),
        queryIds: this.repository.getQueryIds(),
        languageIds: this.repository.getLanguageIds(),
        typeIds: this.repository.getTypeIds(),
        types: this.repository.getTypes(),
        getQuerySource: (id) => this.repository.getQuerySource(id),
        describeRecipe: (id) =>
          describeCache.get(id) ?? this.repository.findById(id),
        readScmFile: (rel) => {
          const candidates = [
            path.isAbsolute(rel) ? rel : path.join(docDir, rel),
            workspaceRoot ? path.join(workspaceRoot, rel) : undefined,
            workspaceRoot
              ? path.join(workspaceRoot, '.codemod', rel)
              : undefined,
            workspaceRoot
              ? path.join(
                  workspaceRoot,
                  '.codemod',
                  'queries',
                  path.basename(rel)
                )
              : undefined,
          ].filter((p): p is string => Boolean(p));
          for (const candidate of candidates) {
            try {
              if (fs.existsSync(candidate) && fs.statSync(candidate).isFile()) {
                return fs.readFileSync(candidate, 'utf8');
              }
            } catch {
              // ignore
            }
          }
          return undefined;
        },
      },
    });

    if (!suggestions) {
      return undefined;
    }
    return suggestions.map((s) => toVscodeItem(s, position.line));
  }
}

export function resolveWorkspacePath(
  workspaceRoot: string,
  relativeOrAbs: string
): string {
  if (path.isAbsolute(relativeOrAbs)) {
    return relativeOrAbs;
  }
  return path.join(workspaceRoot, relativeOrAbs);
}

export { hostMethodByName };
