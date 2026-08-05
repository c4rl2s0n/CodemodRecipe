import * as vscode from 'vscode';
import { COMMANDS } from '../../constants';
import {
  documentTopLevelId,
  firstTopLevelQueryLine,
} from '../recipeHelpers';
import { findEditPathNearLine } from '../../queryTools/recipeYamlExtract';

/**
 * Declarative CodeLens: recipe actions + Query Tools / edit.path navigation.
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
          title: 'Open in Query Tools',
          command: COMMANDS.queryToolsOpenFromRecipe,
          arguments: [document, queryLine],
        })
      );
    }

    // path: under edit: — Go to edit path when present
    for (let i = 0; i < document.lineCount; i++) {
      const text = document.lineAt(i).text;
      if (!/^\s*path:\s*.+/.test(text)) {
        continue;
      }
      // only under edit blocks
      if (!findEditPathNearLine(document, i)) {
        continue;
      }
      const range = new vscode.Range(i, 0, i, text.length);
      lenses.push(
        new vscode.CodeLens(range, {
          title: 'Go to edit path',
          command: COMMANDS.queryToolsGoToEditPath,
          arguments: [document, i],
        })
      );
    }

    return lenses;
  }
}
