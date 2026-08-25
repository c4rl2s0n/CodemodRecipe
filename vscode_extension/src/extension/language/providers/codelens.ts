import * as vscode from 'vscode';
import { COMMANDS } from '../../constants';
import { collectCodeLensDescriptors } from '../navigationLogic';

/**
 * CodeLens on Luau recipe `id = "..."` and first `query =` line.
 */
export class RecipeCodeLensProvider implements vscode.CodeLensProvider {
  constructor(private readonly isUnderCodemod: (uri: vscode.Uri) => boolean) {}

  provideCodeLenses(document: vscode.TextDocument): vscode.CodeLens[] {
    if (!this.isUnderCodemod(document.uri)) {
      return [];
    }
    return collectCodeLensDescriptors(document, {
      openInRecipeRunner: COMMANDS.openInRecipeRunner,
      copyInvokeKeybinding: COMMANDS.copyInvokeKeybinding,
      assignToSlot: COMMANDS.assignToSlot,
      queryToolsOpenFromRecipe: COMMANDS.queryToolsOpenFromRecipe,
    }).map((desc) => {
      const text = document.lineAt(desc.line).text;
      const range = new vscode.Range(desc.line, 0, desc.line, text.length);
      const args =
        desc.command === COMMANDS.queryToolsOpenFromRecipe
          ? [document, desc.arguments[0]]
          : desc.arguments;
      return new vscode.CodeLens(range, {
        title: desc.title,
        command: desc.command,
        arguments: args,
      });
    });
  }
}
