export type KeybindingInvokeMode = 'auto' | 'run' | 'open';

export function formatInvokeKeybindingJson(
  recipeId: string,
  mode: KeybindingInvokeMode = 'auto'
): string {
  return JSON.stringify(
    {
      key: 'ctrl+shift+i 1',
      command: 'codemodRecipe.invoke',
      args: { recipeId, mode },
    },
    null,
    2
  );
}

export function formatSlotKeybindingJson(
  slot: string,
  mode: KeybindingInvokeMode = 'auto'
): string {
  const prefix = mode === 'open' ? 'ctrl+shift+t' : 'ctrl+shift+i';
  return JSON.stringify(
    {
      key: `${prefix} ${slot}`,
      command: 'codemodRecipe.invokeSlot',
      args: { slot, mode },
    },
    null,
    2
  );
}
