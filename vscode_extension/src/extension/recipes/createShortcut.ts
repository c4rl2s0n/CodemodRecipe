import * as vscode from 'vscode';
import type { ExtensionConfig } from '../config/extensionConfig';
import type { RecipeRepository } from './recipeRepository';
import {
  formatSlotKeybindingJson,
  type InvokeMode,
} from './recipeInvoke';
import type { SlotConfig } from './recipeSlots';

export async function createShortcutForRecipe(
  deps: {
    config: ExtensionConfig;
    repository: RecipeRepository;
  },
  recipeId: string,
  fixedArgs?: Record<string, string>
): Promise<void> {
  const recipe = deps.repository.findById(recipeId);
  if (!recipe) {
    vscode.window.showWarningMessage(
      `Codemod Recipe: unknown recipe id "${recipeId}".`
    );
    return;
  }

  const slot = await vscode.window.showInputBox({
    prompt: 'Slot id (any character key, e.g. 1, b, c)',
    placeHolder: 'b',
    validateInput: (value) =>
      value.trim() ? undefined : 'Slot id is required',
  });
  if (!slot?.trim()) {
    return;
  }
  const normalized = slot.trim();

  const modePick = await vscode.window.showQuickPick(
    [
      { label: 'auto', description: 'Apply when args complete; else open runner' },
      { label: 'open', description: 'Always open Recipe Runner' },
      { label: 'run', description: 'Apply or warn + open if incomplete' },
    ],
    { placeHolder: 'Invoke mode for this slot' }
  );
  if (!modePick) {
    return;
  }
  const mode = modePick.label as InvokeMode;

  let args = fixedArgs ?? {};
  if (Object.keys(args).length > 0) {
    const lock = await vscode.window.showQuickPick(
      [
        {
          label: 'Lock current args',
          description: Object.entries(args)
            .map(([k, v]) => `${k}=${v}`)
            .join(', '),
        },
        { label: 'Recipe only (no fixed args)', description: 'Derive args at invoke time' },
      ],
      { placeHolder: 'Store fixed arguments on this slot?' }
    );
    if (!lock) {
      return;
    }
    if (lock.label.startsWith('Recipe only')) {
      args = {};
    }
  }

  const config: SlotConfig =
    mode === 'auto' && Object.keys(args).length === 0
      ? recipeId
      : {
          recipeId,
          mode,
          ...(Object.keys(args).length > 0 ? { args } : {}),
        };

  await deps.config.updateSlot(normalized, config);

  const copy = await vscode.window.showInformationMessage(
    `Codemod Recipe: assigned slot \`${normalized}\` → ${recipeId}`,
    'Copy run keybinding',
    'Copy open keybinding'
  );
  if (copy === 'Copy run keybinding') {
    await vscode.env.clipboard.writeText(
      formatSlotKeybindingJson(normalized, 'auto')
    );
  } else if (copy === 'Copy open keybinding') {
    await vscode.env.clipboard.writeText(
      formatSlotKeybindingJson(normalized, 'open')
    );
  }
}
