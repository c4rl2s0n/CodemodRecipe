import * as vscode from 'vscode';
import { CONFIG } from '../constants';

export class ExtensionConfig {
  get hostEntrypoint(): string {
    return (
      vscode.workspace
        .getConfiguration(CONFIG.section)
        .get<string>(CONFIG.hostEntrypoint) ?? ''
    );
  }

  get dartPath(): string {
    return (
      vscode.workspace
        .getConfiguration(CONFIG.section)
        .get<string>(CONFIG.dartPath) || 'dart'
    );
  }

  get performanceLogging(): boolean {
    return (
      vscode.workspace
        .getConfiguration(CONFIG.section)
        .get<boolean>(CONFIG.performanceLogging) ?? false
    );
  }

  get autoPreview(): boolean {
    return (
      vscode.workspace
        .getConfiguration(CONFIG.section)
        .get<boolean>(CONFIG.autoPreview) ?? true
    );
  }

  get autoPreviewDebounceMs(): number {
    const value =
      vscode.workspace
        .getConfiguration(CONFIG.section)
        .get<number>(CONFIG.autoPreviewDebounceMs) ?? 400;
    return Math.min(2000, Math.max(100, value));
  }

  async updateHostEntrypoint(value: string): Promise<void> {
    await vscode.workspace
      .getConfiguration(CONFIG.section)
      .update(CONFIG.hostEntrypoint, value, vscode.ConfigurationTarget.Workspace);
  }
}
