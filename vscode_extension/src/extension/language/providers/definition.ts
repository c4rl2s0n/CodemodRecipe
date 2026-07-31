import * as fs from 'fs';
import * as path from 'path';
import * as vscode from 'vscode';
import type { ExtensionConfig } from '../../config/extensionConfig';
import type { RecipeRepository } from '../../recipes/recipeRepository';
import {
  matchRecipeReference,
  matchTemplateFile,
} from '../recipeHelpers';

function resolveWorkspacePath(
  workspaceRoot: string,
  relativeOrAbs: string
): string {
  if (path.isAbsolute(relativeOrAbs)) {
    return relativeOrAbs;
  }
  return path.join(workspaceRoot, relativeOrAbs);
}

/**
 * Resolve template paths recipe-local first, then under codemod root
 * (mirrors host resource_path policy for go-to).
 */
export function resolveTemplatePath(
  workspaceRoot: string,
  codemodRoot: string,
  templateFile: string,
  recipeSourceFile?: string
): string | undefined {
  const candidates: string[] = [];
  if (recipeSourceFile) {
    const recipeDir = path.dirname(
      path.isAbsolute(recipeSourceFile)
        ? recipeSourceFile
        : path.join(workspaceRoot, recipeSourceFile)
    );
    candidates.push(path.join(recipeDir, templateFile));
  }
  candidates.push(path.join(workspaceRoot, codemodRoot, templateFile));
  for (const abs of candidates) {
    if (fs.existsSync(abs)) {
      return abs;
    }
  }
  return undefined;
}

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
    const recipeRef = matchRecipeReference(line, position.character);
    if (recipeRef) {
      const target = this.repository.findById(recipeRef);
      if (target?.sourceFile) {
        const abs = resolveWorkspacePath(
          this.config.workspaceRoot,
          target.sourceFile
        );
        return new vscode.Location(
          vscode.Uri.file(abs),
          new vscode.Position(0, 0)
        );
      }
    }

    const templateFile = matchTemplateFile(line, position.character);
    if (templateFile) {
      const topId = document
        .getText()
        .match(/^id:\s*['"]?([A-Za-z_][\w./-]*)['"]?\s*$/m)?.[1];
      const recipe = topId ? this.repository.findById(topId) : undefined;
      const abs = resolveTemplatePath(
        this.config.workspaceRoot,
        this.config.codemodRoot,
        templateFile,
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
