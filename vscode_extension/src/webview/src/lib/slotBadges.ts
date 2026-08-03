/** Documented default chord hints for sample keybinding prefixes. */
export function slotBadgeLabel(slot: string): string {
  return `slot:${slot}`;
}

export function slotBadgeTitle(slot: string): string {
  return `Slot ${slot} (hint: Ctrl+Shift+I ${slot} / Ctrl+Shift+T ${slot})`;
}

export function slotsForRecipe(
  slotsByRecipe: Record<string, string[]> | undefined,
  recipeId: string
): string[] {
  return slotsByRecipe?.[recipeId] ?? [];
}
