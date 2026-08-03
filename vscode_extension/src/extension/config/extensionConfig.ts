import * as vscode from 'vscode';
import { CONFIG } from '../constants';
import * as path from 'path';
import {
  parseSlotConfig,
  serializeSlotConfig,
  type SlotConfig,
} from '../recipes/recipeSlots';

export class ExtensionConfig {
  get workspaceRoot(): string {
    const configuredRoot = vscode.workspace
      .getConfiguration(CONFIG.section)
      .get<string>(CONFIG.workspaceRoot) || '';

    // If workspaceRoot is configured, use it (must be absolute path)
    if (configuredRoot && path.isAbsolute(configuredRoot)) {
      return configuredRoot;
    }

    // Otherwise, use the currently open VSCode workspace folder
    return vscode.workspace.workspaceFolders?.[0]?.uri?.fsPath || '.';
  }

  get codemodRoot(): string {
    return (
      vscode.workspace
        .getConfiguration(CONFIG.section)
        .get<string>(CONFIG.codemodRoot) ?? '.codemod'
    );
  }

  get performanceLogging(): boolean {
    return (
      vscode.workspace
        .getConfiguration(CONFIG.section)
        .get<boolean>(CONFIG.performanceLogging) ?? false
    );
  }

  get autoPreviewDebounceMs(): number {
    const value =
      vscode.workspace
        .getConfiguration(CONFIG.section)
        .get<number>(CONFIG.autoPreviewDebounceMs) ?? 400;
    return Math.min(2000, Math.max(100, value));
  }

  get previewSnippetLines(): number {
    const value =
      vscode.workspace
        .getConfiguration(CONFIG.section)
        .get<number>(CONFIG.previewSnippetLines) ?? 5;
    return Math.min(20, Math.max(1, value));
  }

  /** Slot id → recipe id (string sugar only; structured slots expose recipeId). */
  get slots(): Record<string, string> {
    const out: Record<string, string> = {};
    for (const [key, config] of Object.entries(this.structuredSlots)) {
      out[key] = typeof config === 'string' ? config : config.recipeId;
    }
    return out;
  }

  /** Full slot map including structured `{ recipeId, mode?, args? }`. */
  get structuredSlots(): Record<string, SlotConfig> {
    const raw =
      vscode.workspace
        .getConfiguration(CONFIG.section)
        .get<Record<string, unknown>>(CONFIG.slots) ?? {};
    const out: Record<string, SlotConfig> = {};
    for (const [key, value] of Object.entries(raw)) {
      const parsed = parseSlotConfig(value);
      if (parsed) {
        out[key.trim()] = parsed;
      }
    }
    return out;
  }

  get shortcutConfirmApply(): boolean {
    return (
      vscode.workspace
        .getConfiguration(CONFIG.section)
        .get<boolean>(CONFIG.shortcutConfirmApply) ?? false
    );
  }

  async updateCodemodRoot(value: string): Promise<void> {
    await vscode.workspace
      .getConfiguration(CONFIG.section)
      .update(CONFIG.codemodRoot, value, vscode.ConfigurationTarget.Workspace);
  }

  async updateSlot(
    slot: string,
    recipeIdOrConfig: string | SlotConfig,
    target: vscode.ConfigurationTarget = vscode.ConfigurationTarget.Workspace
  ): Promise<void> {
    const config: SlotConfig =
      typeof recipeIdOrConfig === 'string'
        ? recipeIdOrConfig
        : recipeIdOrConfig;
    const next: Record<string, string | Record<string, unknown>> = {};
    for (const [key, value] of Object.entries(this.structuredSlots)) {
      next[key] = serializeSlotConfig(value);
    }
    next[slot] = serializeSlotConfig(config);
    await vscode.workspace
      .getConfiguration(CONFIG.section)
      .update(CONFIG.slots, next, target);
  }
}
