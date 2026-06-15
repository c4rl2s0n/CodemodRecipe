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

  get recipesDirectory(): string {
    return (
      vscode.workspace
        .getConfiguration(CONFIG.section)
        .get<string>(CONFIG.recipesDirectory) ?? '.codemod/recipes'
    );
  }

  get templatesRoot(): string {
    return (
      vscode.workspace
        .getConfiguration(CONFIG.section)
        .get<string>(CONFIG.templatesRoot) ?? '.codemod/templates'
    );
  }

  get emptyConstructorStyle(): 'named' | 'positional' {
    const value =
      vscode.workspace
        .getConfiguration(CONFIG.section)
        .get<string>(CONFIG.emptyConstructorStyle) ?? 'named';
    return value === 'positional' ? 'positional' : 'named';
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

  async updateHostEntrypoint(value: string): Promise<void> {
    await vscode.workspace
      .getConfiguration(CONFIG.section)
      .update(CONFIG.hostEntrypoint, value, vscode.ConfigurationTarget.Workspace);
  }
}
