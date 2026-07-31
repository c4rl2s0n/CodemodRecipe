import * as fs from 'fs';
import * as path from 'path';
import * as vscode from 'vscode';
import type { ExtensionConfig } from '../../config/extensionConfig';
import type { RecipeRepository } from '../../recipes/recipeRepository';
import { loadKeywordDocs, lookupKeywordHover } from '../keywordDocs';
import {
  matchRecipeReference,
  matchTemplateFile,
} from '../recipeHelpers';

export class RecipeHoverProvider implements vscode.HoverProvider {
  private readonly keywordDocs;

  constructor(
    extensionUri: vscode.Uri,
    private readonly repository: RecipeRepository,
    private readonly config: ExtensionConfig,
    private readonly isUnderCodemod: (uri: vscode.Uri) => boolean
  ) {
    this.keywordDocs = loadKeywordDocs(extensionUri);
  }

  async provideHover(
    document: vscode.TextDocument,
    position: vscode.Position
  ): Promise<vscode.Hover | undefined> {
    if (!this.isUnderCodemod(document.uri)) {
      return;
    }
    const line = document.lineAt(position.line).text;

    const keywordDesc = lookupKeywordHover(
      this.keywordDocs,
      line,
      position.character
    );
    if (keywordDesc) {
      return new vscode.Hover(new vscode.MarkdownString(keywordDesc));
    }

    const recipeRef = matchRecipeReference(line, position.character);
    if (recipeRef) {
      const described =
        (await this.repository.describeCached(recipeRef)) ??
        this.repository.findById(recipeRef);
      if (!described) {
        return;
      }
      const args =
        described.args.length === 0
          ? '_No args_'
          : described.args
              .map((arg) => {
                const req = arg.required ? ' (required)' : '';
                return `- \`${arg.name}\`${req}${arg.help ? `: ${arg.help}` : ''}`;
              })
              .join('\n');
      return new vscode.Hover(
        new vscode.MarkdownString(
          `**${described.name}** (\`${described.id}\`)\n\n${described.description}\n\n**Args**\n${args}`
        )
      );
    }

    const templateFile = matchTemplateFile(line, position.character);
    if (templateFile) {
      const abs = path.join(
        this.config.workspaceRoot,
        this.config.codemodRoot,
        templateFile
      );
      try {
        const content = fs.readFileSync(abs, 'utf8');
        const preview = content.split(/\r?\n/).slice(0, 20).join('\n');
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
