import * as vscode from 'vscode';
import { COMMANDS } from '../../constants';
import {
  documentTopLevelId,
  firstTopLevelQueryLine,
} from '../recipeHelpers';

/**
 * Declarative CodeLens: top-level recipe id only + one test-query lens.
 */
export class RecipeCodeLensProvider implements vscode.CodeLensProvider {
  constructor(private readonly isUnderCodemod: (uri: vscode.Uri) => boolean) {}

  provideCodeLenses(document: vscode.TextDocument): vscode.CodeLens[] {
    if (!this.isUnderCodemod(document.uri)) {
      return [];
    }
    const lenses: vscode.CodeLens[] = [];
    const recipeId = documentTopLevelId(document);
    if (!recipeId) {
      return [];
    }

    for (let i = 0; i < Math.min(document.lineCount, 40); i++) {
      const text = document.lineAt(i).text;
      const idMatch = text.match(/^id:\s*['"]?([A-Za-z_][\w./-]*)['"]?\s*$/);
      if (idMatch && idMatch[1] === recipeId) {
        const range = new vscode.Range(i, 0, i, text.length);
        lenses.push(
          new vscode.CodeLens(range, {
            title: 'Open in Recipe Runner',
            command: COMMANDS.openInRecipeRunner,
            arguments: [recipeId],
          }),
          new vscode.CodeLens(range, {
            title: 'Copy invoke keybinding',
            command: COMMANDS.copyInvokeKeybinding,
            arguments: [recipeId],
          }),
          new vscode.CodeLens(range, {
            title: 'Assign to slot…',
            command: COMMANDS.assignToSlot,
            arguments: [recipeId],
          })
        );
        break;
      }
    }

    const queryLine = firstTopLevelQueryLine(document);
    if (queryLine !== undefined) {
      const text = document.lineAt(queryLine).text;
      const range = new vscode.Range(queryLine, 0, queryLine, text.length);
      lenses.push(
        new vscode.CodeLens(range, {
          title: 'Test query on file…',
          command: COMMANDS.testQueryOnFile,
          arguments: [recipeId],
        })
      );
    }

    return lenses;
  }
}
