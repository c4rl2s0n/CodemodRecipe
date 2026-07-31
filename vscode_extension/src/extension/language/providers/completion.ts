import * as path from 'path';
import * as vscode from 'vscode';
import type { HostBridge } from '../../host/hostBridge';
import type { RecipeRepository } from '../../recipes/recipeRepository';
import { collectRecipeIdCompletions, recipeIdCompletionContext } from '../../../shared';
import type { DslSurface } from '../dslSurface';
import { remainingChildKeys, resolveContainerId } from '../dslSurface';
import { resolveYamlContext, lineSourceFromDocument } from '../yamlContext';
import {
  collectSetWithKeys,
  findNearbyRecipeStepId,
  inWithBlock,
  isUnderRecipeMapping,
  parseArgNames,
} from '../recipeHelpers';

export class RecipeCompletionProvider implements vscode.CompletionItemProvider {
  constructor(
    private readonly repository: RecipeRepository,
    private readonly bridge: HostBridge,
    private readonly surface: DslSurface,
    private readonly isUnderCodemod: (uri: vscode.Uri) => boolean
  ) {}

  async provideCompletionItems(
    document: vscode.TextDocument,
    position: vscode.Position
  ): Promise<vscode.CompletionItem[] | undefined> {
    if (!this.isUnderCodemod(document.uri)) {
      return;
    }

    const line = document.lineAt(position.line).text;
    const before = line.slice(0, position.character);
    const text = document.getText();
    const ctx = resolveYamlContext(
      lineSourceFromDocument(document),
      position.line,
      position.character
    );

    if (/var\.\w*$/.test(before)) {
      return this.repository.getVarIds().map((id) => {
        const item = new vscode.CompletionItem(id, vscode.CompletionItemKind.Variable);
        item.insertText = id;
        return item;
      });
    }

    if (/map\.\w*$/.test(before)) {
      return this.repository.getMapIds().map((id) => {
        const item = new vscode.CompletionItem(id, vscode.CompletionItemKind.Enum);
        item.insertText = id;
        return item;
      });
    }

    if (/\{\{\s*[\w.]*$/.test(before)) {
      const args = parseArgNames(text);
      const items: vscode.CompletionItem[] = args.map((name) => {
        const item = new vscode.CompletionItem(name, vscode.CompletionItemKind.Field);
        item.detail = 'recipe arg';
        return item;
      });
      for (const id of this.repository.getVarIds()) {
        const item = new vscode.CompletionItem(
          `var.${id}`,
          vscode.CompletionItemKind.Variable
        );
        item.insertText = `var.${id}`;
        items.push(item);
      }
      for (const id of this.repository.getMapIds()) {
        const item = new vscode.CompletionItem(
          `map.${id}`,
          vscode.CompletionItemKind.Enum
        );
        item.insertText = `map.${id}|`;
        item.detail = 'map filter (append key)';
        items.push(item);
      }
      return items;
    }

    if (
      /^\s*language:\s*[\w-]*$/i.test(before.trimEnd()) ||
      /language:\s*[\w-]*$/i.test(before)
    ) {
      return this.repository.getLanguageIds().map((id) => {
        return new vscode.CompletionItem(id, vscode.CompletionItemKind.Value);
      });
    }

    if (ctx.positionKind === 'key') {
      const containerId = resolveContainerId(
        this.surface,
        ctx.parentWire,
        ctx.path
      );
      if (containerId) {
        const container = this.surface.containers[containerId];
        if (!container?.mapValue) {
          const keys = remainingChildKeys(
            this.surface,
            containerId,
            ctx.siblingKeys
          );
          if (keys.length > 0) {
            const typed = before.trim();
            return keys
              .filter((k) => !typed || k.startsWith(typed))
              .map((key) => {
                const item = new vscode.CompletionItem(
                  key,
                  vscode.CompletionItemKind.Property
                );
                item.insertText = `${key}: `;
                item.sortText = `0_${key}`;
                item.detail = `codemod (${containerId})`;
                return item;
              });
          }
        }
      }
    }

    const recipeIdPrefix = recipeIdCompletionContext(before);
    if (
      recipeIdPrefix &&
      (/(?:^|\s)recipe:\s*['"]?[\w./-]*$/.test(before) ||
        /^\s*id:\s*['"]?[\w./-]*$/.test(before) ||
        isUnderRecipeMapping(document, position.line))
    ) {
      const segmentStart =
        recipeIdPrefix.start + recipeIdPrefix.typed.lastIndexOf('.') + 1;
      const range = new vscode.Range(
        position.line,
        segmentStart,
        position.line,
        position.character
      );
      return collectRecipeIdCompletions(
        this.repository.getRecipes().map((recipe) => recipe.id),
        recipeIdPrefix.typed
      ).map((completion) => {
        const item = new vscode.CompletionItem(
          completion.label,
          completion.hasChildren
            ? vscode.CompletionItemKind.Module
            : vscode.CompletionItemKind.Reference
        );
        item.insertText = completion.insertText;
        item.range = range;
        item.detail = completion.hasChildren
          ? `Recipe group: ${completion.fullPath}`
          : `Recipe id: ${completion.fullPath}`;
        item.sortText = `1_${completion.label}`;
        return item;
      });
    }

    const enumField = before.match(/^\s*([A-Za-z_][\w]*)\s*:\s*['"]?([\w]*)$/);
    if (enumField && ctx.positionKind === 'value') {
      const field = enumField[1];
      for (const [enumId, values] of Object.entries(this.surface.enums)) {
        if (
          field === enumId ||
          (field === 'anchor' && enumId === 'anchor') ||
          (field === 'ifExists' && enumId === 'ifExists') ||
          (field === 'ifMissing' && enumId === 'ifMissing') ||
          (field === 'inputKind' && enumId === 'inputKind') ||
          (field === 'extract' && enumId === 'extract') ||
          (field === 'onNoMatch' && enumId === 'onNoMatch') ||
          (field === 'onManyMatches' && enumId === 'onManyMatches') ||
          (field === 'kind' && enumId === 'explorerMenuKind')
        ) {
          const typed = enumField[2] ?? '';
          return values
            .filter((v) => v.startsWith(typed))
            .map((v) => {
              const item = new vscode.CompletionItem(
                v,
                vscode.CompletionItemKind.EnumMember
              );
              item.sortText = `0_${v}`;
              return item;
            });
        }
      }
    }

    const childRecipeId = findNearbyRecipeStepId(document, position.line);
    if (
      childRecipeId &&
      (inWithBlock(document, position.line) ||
        (ctx.parentWire === 'with' && ctx.positionKind === 'key'))
    ) {
      try {
        await this.bridge.ensureHost();
      } catch {
        // ignore
      }
      const described = await this.repository.describeCached(childRecipeId);
      if (described) {
        const alreadySet = collectSetWithKeys(document, position.line);
        const remaining = described.args
          .filter((arg) => !alreadySet.has(arg.name))
          .sort((a, b) => {
            if (a.required !== b.required) {
              return a.required ? -1 : 1;
            }
            return a.name.localeCompare(b.name);
          });
        return remaining.map((arg) => {
          const item = new vscode.CompletionItem(
            arg.name,
            vscode.CompletionItemKind.Property
          );
          const parts: string[] = [];
          if (arg.required) {
            parts.push('required');
          }
          if (arg.help) {
            parts.push(arg.help);
          }
          item.detail = parts.length ? parts.join(' — ') : undefined;
          item.insertText = `${arg.name}: `;
          item.sortText = `${arg.required ? '0' : '1'}_${arg.name}`;
          return item;
        });
      }
    }

    return undefined;
  }
}

export function resolveWorkspacePath(
  workspaceRoot: string,
  relativeOrAbs: string
): string {
  if (path.isAbsolute(relativeOrAbs)) {
    return relativeOrAbs;
  }
  return path.join(workspaceRoot, relativeOrAbs);
}
