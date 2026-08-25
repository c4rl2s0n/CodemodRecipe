import * as fs from 'fs';
import * as path from 'path';
import * as vscode from 'vscode';
import type { ExtensionConfig } from '../../config/extensionConfig';
import type { RecipeRepository } from '../../recipes/recipeRepository';
import { resolveHoverContent } from '../navigationLogic';
import { matchIdReference, matchTemplatePath } from '../luauContext';

export class RecipeHoverProvider implements vscode.HoverProvider {
  constructor(
    private readonly repository: RecipeRepository,
    private readonly config: ExtensionConfig,
    private readonly isUnderCodemod: (uri: vscode.Uri) => boolean
  ) {}

  async provideHover(
    document: vscode.TextDocument,
    position: vscode.Position
  ): Promise<vscode.Hover | undefined> {
    if (!this.isUnderCodemod(document.uri)) {
      return;
    }
    const line = document.lineAt(position.line).text;
    const wordRange = document.getWordRangeAtPosition(
      position,
      /[A-Za-z_]\w*/
    );
    const word = wordRange ? document.getText(wordRange) : undefined;

    const recipeRef = matchIdReference(line, position.character);
    const describedRecipe =
      recipeRef && (recipeRef.kind === 'run' || recipeRef.kind === 'id')
        ? await this.repository.describeCached(recipeRef.id)
        : undefined;

    const content = resolveHoverContent(
      line,
      position.character,
      word,
      (id) => {
        if (describedRecipe?.id === id) {
          return describedRecipe;
        }
        return this.repository.findById(id);
      },
      (id) => this.repository.getQuerySource(id)
    );

    if (content?.kind === 'markdown') {
      return new vscode.Hover(new vscode.MarkdownString(content.text));
    }
    if (content?.kind === 'code') {
      const md = new vscode.MarkdownString();
      if (content.preface) {
        md.appendMarkdown(content.preface);
      }
      md.appendCodeblock(content.text, content.language);
      return new vscode.Hover(md);
    }

    const templateFile = matchTemplatePath(line, position.character);
    if (templateFile) {
      const abs = path.join(
        this.config.workspaceRoot,
        this.config.codemodRoot,
        templateFile
      );
      try {
        const fileContent = fs.readFileSync(abs, 'utf8');
        const preview = fileContent.split(/\r?\n/).slice(0, 20).join('\n');
        const md = new vscode.MarkdownString();
        md.appendCodeblock(preview, 'plaintext');
        return new vscode.Hover(md);
      } catch {
        return new vscode.Hover(`Template not found: ${templateFile}`);
      }
    }
    return undefined;
  }
}
