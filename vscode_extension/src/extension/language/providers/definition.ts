import * as vscode from 'vscode';
import type { ExtensionConfig } from '../../config/extensionConfig';
import type { RecipeRepository } from '../../recipes/recipeRepository';
import {
  resolveDefinitionTarget,
  resolveTemplatePath,
  resolveWorkspacePath,
} from '../navigationLogic';

export { resolveTemplatePath, resolveWorkspacePath };

export class RecipeDefinitionProvider implements vscode.DefinitionProvider {
  constructor(
    private readonly repository: RecipeRepository,
    private readonly config: ExtensionConfig,
    private readonly isUnderCodemod: (uri: vscode.Uri) => boolean
  ) {}

  provideDefinition(
    document: vscode.TextDocument,
    position: vscode.Position
  ): vscode.ProviderResult<vscode.Definition> {
    if (!this.isUnderCodemod(document.uri)) {
      return;
    }
    const line = document.lineAt(position.line).text;
    const target = resolveDefinitionTarget(
      line,
      position.character,
      document.getText(),
      (id) => this.repository.findById(id)
    );
    if (target?.kind === 'recipe') {
      const abs = resolveWorkspacePath(
        this.config.workspaceRoot,
        target.sourceFile
      );
      return new vscode.Location(
        vscode.Uri.file(abs),
        new vscode.Position(0, 0)
      );
    }
    if (target?.kind === 'template') {
      const topId = document
        .getText()
        .match(/\bid\s*=\s*['"]([A-Za-z_][\w./-]*)['"]/)?.[1];
      const recipe = topId ? this.repository.findById(topId) : undefined;
      const abs = resolveTemplatePath(
        this.config.workspaceRoot,
        this.config.codemodRoot,
        target.path,
        recipe?.sourceFile ?? undefined
      );
      if (abs) {
        return new vscode.Location(
          vscode.Uri.file(abs),
          new vscode.Position(0, 0)
        );
      }
    }
    return undefined;
  }
}
