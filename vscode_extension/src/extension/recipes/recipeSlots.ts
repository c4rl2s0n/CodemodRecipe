export type SlotInvokeMode = 'auto' | 'run' | 'open';

/** Settings value: recipe id string, or structured shortcut metadata. */
export type SlotConfig =
  | string
  | {
      recipeId: string;
      mode?: SlotInvokeMode;
      args?: Record<string, string>;
    };

export interface ResolvedSlot {
  slot: string;
  recipeId: string;
  mode: SlotInvokeMode;
  args: Record<string, string>;
}

export function parseSlotConfig(raw: unknown): SlotConfig | undefined {
  if (typeof raw === 'string') {
    const trimmed = raw.trim();
    return trimmed ? trimmed : undefined;
  }
  if (raw && typeof raw === 'object' && !Array.isArray(raw)) {
    const obj = raw as Record<string, unknown>;
    const recipeId =
      typeof obj.recipeId === 'string' ? obj.recipeId.trim() : '';
    if (!recipeId) {
      return undefined;
    }
    const mode =
      obj.mode === 'auto' || obj.mode === 'run' || obj.mode === 'open'
        ? obj.mode
        : undefined;
    const args: Record<string, string> = {};
    if (obj.args && typeof obj.args === 'object' && !Array.isArray(obj.args)) {
      for (const [k, v] of Object.entries(obj.args as Record<string, unknown>)) {
        if (typeof v === 'string') {
          args[k] = v;
        }
      }
    }
    return {
      recipeId,
      ...(mode ? { mode } : {}),
      ...(Object.keys(args).length > 0 ? { args } : {}),
    };
  }
  return undefined;
}

export function resolveSlot(
  slot: string,
  config: SlotConfig | undefined
): ResolvedSlot | undefined {
  if (config == null) {
    return undefined;
  }
  if (typeof config === 'string') {
    return { slot, recipeId: config, mode: 'auto', args: {} };
  }
  return {
    slot,
    recipeId: config.recipeId,
    mode: config.mode ?? 'auto',
    args: config.args ?? {},
  };
}

export function recipeIdFromSlotConfig(config: SlotConfig): string {
  return typeof config === 'string' ? config : config.recipeId;
}

/** Invert slots → recipeId → slot ids (stable sorted). */
export function slotsByRecipeId(
  slots: Record<string, SlotConfig>
): Record<string, string[]> {
  const out: Record<string, string[]> = {};
  for (const [slot, config] of Object.entries(slots)) {
    const recipeId = recipeIdFromSlotConfig(config);
    if (!out[recipeId]) {
      out[recipeId] = [];
    }
    out[recipeId].push(slot);
  }
  for (const ids of Object.values(out)) {
    ids.sort();
  }
  return out;
}

/** Documented default chord prefixes for our sample keybindings (hints only). */
export function hintChordsForSlot(slot: string): { run: string; open: string } {
  return {
    run: `Ctrl+Shift+I ${slot}`,
    open: `Ctrl+Shift+T ${slot}`,
  };
}

export function serializeSlotConfig(config: SlotConfig): string | Record<string, unknown> {
  if (typeof config === 'string') {
    return config;
  }
  const out: Record<string, unknown> = { recipeId: config.recipeId };
  if (config.mode) {
    out.mode = config.mode;
  }
  if (config.args && Object.keys(config.args).length > 0) {
    out.args = config.args;
  }
  return out;
}
