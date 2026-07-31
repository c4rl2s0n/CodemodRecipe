import * as fs from 'fs';
import * as path from 'path';
import * as vscode from 'vscode';
import { COMMANDS } from '../constants';
import type { ExtensionConfig } from '../config/extensionConfig';
import type { HostBridge } from '../host/hostBridge';
import type { RecipeRepository } from '../recipes/recipeRepository';
import { collectRecipeIdCompletions, recipeIdCompletionContext } from '../../shared';
import { loadKeywordDocs, lookupKeywordHover } from './keywordDocs';

const YAML_SELECTOR: vscode.DocumentSelector = [
  { language: 'yaml', pattern: '**/.codemod/**/*.{yaml,yml}' },
  { language: 'yaml', scheme: 'file', pattern: '**/.codemod/**' },
];

export function registerRecipeLanguageSupport(
  context: vscode.ExtensionContext,
  repository: RecipeRepository,
  bridge: HostBridge,
  config: ExtensionConfig
): void {
  const codemodRootAbs = () =>
    path.join(config.workspaceRoot, config.codemodRoot);

  const isUnderCodemod = (uri: vscode.Uri): boolean => {
    const root = path.normalize(codemodRootAbs());
    const file = path.normalize(uri.fsPath);
    return file === root || file.startsWith(root + path.sep);
  };

  context.subscriptions.push(
    vscode.languages.registerDefinitionProvider(
      YAML_SELECTOR,
      new RecipeDefinitionProvider(repository, config, isUnderCodemod)
    ),
    vscode.languages.registerCompletionItemProvider(
      YAML_SELECTOR,
      new RecipeCompletionProvider(repository, bridge, isUnderCodemod),
      ':',
      ' ',
      '.',
      '{',
      '"'
    ),
    vscode.languages.registerHoverProvider(
      YAML_SELECTOR,
      new RecipeHoverProvider(context.extensionUri, repository, config, isUnderCodemod)
    ),
    vscode.languages.registerCodeLensProvider(
      YAML_SELECTOR,
      new RecipeCodeLensProvider(isUnderCodemod)
    ),
    vscode.commands.registerCommand(
      COMMANDS.openInRecipeRunner,
      async (recipeId?: string) => {
        if (!recipeId || typeof recipeId !== 'string') {
          return;
        }
        const recipe = repository.findById(recipeId);
        if (!recipe) {
          vscode.window.showWarningMessage(
            `Codemod Recipe: unknown recipe id "${recipeId}"`
          );
          return;
        }
        await vscode.commands.executeCommand(COMMANDS.runRecipe, recipe);
      }
    ),
    vscode.commands.registerCommand(
      COMMANDS.testQueryOnFile,
      async (recipeId?: string) => {
        if (!recipeId || typeof recipeId !== 'string') {
          return;
        }
        const recipe = repository.findById(recipeId);
        if (!recipe) {
          vscode.window.showWarningMessage(
            `Codemod Recipe: unknown recipe id "${recipeId}"`
          );
          return;
        }
        const picked = await vscode.window.showOpenDialog({
          canSelectMany: false,
          canSelectFolders: false,
          openLabel: 'Preview recipe on file',
          defaultUri: vscode.Uri.file(config.workspaceRoot),
        });
        if (!picked?.[0]) {
          return;
        }
        const rel = path.relative(config.workspaceRoot, picked[0].fsPath);
        const fileArg = rel.startsWith('..') ? picked[0].fsPath : rel;
        const fileArgName =
          recipe.args.find((a) => a.name === 'file' || a.inputKind === 'file')
            ?.name ?? 'file';
        await vscode.commands.executeCommand(COMMANDS.runRecipe, recipe, {
          [fileArgName]: fileArg,
        });
      }
    )
  );
}

class RecipeDefinitionProvider implements vscode.DefinitionProvider {
  constructor(
    private readonly repository: RecipeRepository,
    private readonly config: ExtensionConfig,
    private readonly isUnderCodemod: (uri: vscode.Uri) => boolean
  ) {}

  provideDefinition(
    document: vscode.TextDocument,
    position: vscode.Position
  ): vscode.ProviderResult<vscode.Definition> {
    if (!this.isUnderCodemod(document.uri)) {
      return;
    }
    const line = document.lineAt(position.line).text;
    const recipeRef = matchRecipeReference(line, position.character);
    if (recipeRef) {
      const target = this.repository.findById(recipeRef);
      if (target?.sourceFile) {
        const abs = resolveWorkspacePath(
          this.config.workspaceRoot,
          target.sourceFile
        );
        return new vscode.Location(vscode.Uri.file(abs), new vscode.Position(0, 0));
      }
    }

    const templateFile = matchTemplateFile(line, position.character);
    if (templateFile) {
      const abs = path.join(
        this.config.workspaceRoot,
        this.config.codemodRoot,
        templateFile
      );
      if (fs.existsSync(abs)) {
        return new vscode.Location(vscode.Uri.file(abs), new vscode.Position(0, 0));
      }
    }
    return undefined;
  }
}

class RecipeCompletionProvider implements vscode.CompletionItemProvider {
  constructor(
    private readonly repository: RecipeRepository,
    private readonly bridge: HostBridge,
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

    if (/^\s*language:\s*[\w-]*$/i.test(before.trimEnd()) || /language:\s*[\w-]*$/i.test(before)) {
      return this.repository.getLanguageIds().map((id) => {
        const item = new vscode.CompletionItem(id, vscode.CompletionItemKind.Value);
        return item;
      });
    }

    const recipeIdPrefix = recipeIdCompletionContext(before);
    if (
      recipeIdPrefix &&
      (/(?:^|\s)recipe:\s*['"]?[\w./-]*$/.test(before) ||
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
        return item;
      });
    }

    const childRecipeId = findNearbyRecipeStepId(document, position.line);
    if (childRecipeId && inWithBlock(document, position.line)) {
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

class RecipeHoverProvider implements vscode.HoverProvider {
  private readonly keywordDocs;

  constructor(
    extensionUri: vscode.Uri,
    private readonly repository: RecipeRepository,
    private readonly config: ExtensionConfig,
    private readonly isUnderCodemod: (uri: vscode.Uri) => boolean
  ) {
    this.keywordDocs = loadKeywordDocs(extensionUri);
  }

  async provideHover(
    document: vscode.TextDocument,
    position: vscode.Position
  ): Promise<vscode.Hover | undefined> {
    if (!this.isUnderCodemod(document.uri)) {
      return;
    }
    const line = document.lineAt(position.line).text;

    const keywordDesc = lookupKeywordHover(
      this.keywordDocs,
      line,
      position.character
    );
    if (keywordDesc) {
      return new vscode.Hover(new vscode.MarkdownString(keywordDesc));
    }

    const recipeRef = matchRecipeReference(line, position.character);
    if (recipeRef) {
      const described =
        (await this.repository.describeCached(recipeRef)) ??
        this.repository.findById(recipeRef);
      if (!described) {
        return;
      }
      const args =
        described.args.length === 0
          ? '_No args_'
          : described.args
              .map((arg) => {
                const req = arg.required ? ' (required)' : '';
                return `- \`${arg.name}\`${req}${arg.help ? `: ${arg.help}` : ''}`;
              })
              .join('\n');
      return new vscode.Hover(
        new vscode.MarkdownString(
          `**${described.name}** (\`${described.id}\`)\n\n${described.description}\n\n**Args**\n${args}`
        )
      );
    }

    const templateFile = matchTemplateFile(line, position.character);
    if (templateFile) {
      const abs = path.join(
        this.config.workspaceRoot,
        this.config.codemodRoot,
        templateFile
      );
      try {
        const content = fs.readFileSync(abs, 'utf8');
        const preview = content.split(/\r?\n/).slice(0, 20).join('\n');
        const md = new vscode.MarkdownString();
        md.appendCodeblock(preview, 'plaintext');
        return new vscode.Hover(md);
      } catch {
        return new vscode.Hover(`Template not found: ${templateFile}`);
      }
    }
    return undefined;
  }
}

class RecipeCodeLensProvider implements vscode.CodeLensProvider {
  constructor(private readonly isUnderCodemod: (uri: vscode.Uri) => boolean) {}

  provideCodeLenses(document: vscode.TextDocument): vscode.CodeLens[] {
    if (!this.isUnderCodemod(document.uri)) {
      return [];
    }
    const lenses: vscode.CodeLens[] = [];
    const recipeId = documentTopLevelId(document);
    for (let i = 0; i < document.lineCount; i++) {
      const text = document.lineAt(i).text;
      const idMatch = text.match(/^id:\s*['"]?([A-Za-z_][\w./-]*)['"]?\s*$/);
      if (idMatch) {
        const range = new vscode.Range(i, 0, i, text.length);
        lenses.push(
          new vscode.CodeLens(range, {
            title: 'Open in Recipe Runner',
            command: COMMANDS.openInRecipeRunner,
            arguments: [idMatch[1]],
          })
        );
        lenses.push(
          new vscode.CodeLens(range, {
            title: 'Copy invoke keybinding',
            command: COMMANDS.copyInvokeKeybinding,
            arguments: [idMatch[1]],
          })
        );
        lenses.push(
          new vscode.CodeLens(range, {
            title: 'Assign to slot…',
            command: COMMANDS.assignToSlot,
            arguments: [idMatch[1]],
          })
        );
      }
      if (recipeId && /^\s*query:\s*/.test(text)) {
        const range = new vscode.Range(i, 0, i, text.length);
        lenses.push(
          new vscode.CodeLens(range, {
            title: 'Test query on file…',
            command: COMMANDS.testQueryOnFile,
            arguments: [recipeId],
          })
        );
      }
    }
    return lenses;
  }
}

function documentTopLevelId(document: vscode.TextDocument): string | undefined {
  for (let i = 0; i < Math.min(document.lineCount, 40); i++) {
    const text = document.lineAt(i).text;
    const match = text.match(/^id:\s*['"]?([A-Za-z_][\w./-]*)['"]?\s*$/);
    if (match) {
      return match[1];
    }
  }
  return undefined;
}

function resolveWorkspacePath(workspaceRoot: string, relativeOrAbs: string): string {
  if (path.isAbsolute(relativeOrAbs)) {
    return relativeOrAbs;
  }
  return path.join(workspaceRoot, relativeOrAbs);
}

function matchRecipeReference(line: string, character: number): string | undefined {
  const patterns = [
    /\brecipe:\s*['"]?([A-Za-z_][\w./-]*)['"]?/,
    /^\s*id:\s*['"]?([A-Za-z_][\w./-]*)['"]?\s*$/,
  ];
  for (const pattern of patterns) {
    const match = pattern.exec(line);
    if (!match || match.index === undefined) {
      continue;
    }
    const start = match.index + match[0].indexOf(match[1]);
    const end = start + match[1].length;
    if (character >= start && character <= end) {
      return match[1];
    }
  }
  return undefined;
}

function matchTemplateFile(line: string, character: number): string | undefined {
  const match = /templateFile:\s*['"]?([^\s'"]+)['"]?/.exec(line);
  if (!match || match.index === undefined) {
    return undefined;
  }
  const start = match.index + match[0].indexOf(match[1]);
  const end = start + match[1].length;
  if (character >= start && character <= end) {
    return match[1];
  }
  return undefined;
}

function isUnderRecipeMapping(
  document: vscode.TextDocument,
  line: number
): boolean {
  for (let i = line - 1; i >= Math.max(0, line - 12); i--) {
    const text = document.lineAt(i).text;
    if (/^\s*recipe:\s*$/.test(text) || /^\s*-\s*recipe:\s*$/.test(text)) {
      return true;
    }
    if (/^\s*-\s*(edit|create|delete):/.test(text)) {
      return false;
    }
  }
  return false;
}

function findNearbyRecipeStepId(
  document: vscode.TextDocument,
  line: number
): string | undefined {
  for (let i = line; i >= Math.max(0, line - 20); i--) {
    const text = document.lineAt(i).text;
    const inline = text.match(/^\s*(?:-\s*)?recipe:\s*['"]?([A-Za-z_][\w./-]*)['"]?\s*$/);
    if (inline) {
      return inline[1];
    }
    const idLine = text.match(/^\s*id:\s*['"]?([A-Za-z_][\w./-]*)['"]?\s*$/);
    if (idLine && isUnderRecipeMapping(document, i)) {
      return idLine[1];
    }
  }
  return undefined;
}

function inWithBlock(document: vscode.TextDocument, line: number): boolean {
  for (let i = line; i >= Math.max(0, line - 30); i--) {
    const text = document.lineAt(i).text;
    if (/^\s*with:\s*$/.test(text)) {
      return true;
    }
    if (/^\s*(?:-\s*)?(edit|create|delete|recipe):/.test(text) && i < line) {
      return false;
    }
  }
  return false;
}

/**
 * Collect keys already present under the enclosing `with:` mapping.
 * Stops when indentation leaves the `with:` block.
 */
export function collectSetWithKeys(
  document: vscode.TextDocument,
  line: number
): Set<string> {
  const keys = new Set<string>();
  let withIndent = -1;
  let withLine = -1;
  for (let i = line; i >= Math.max(0, line - 40); i--) {
    const text = document.lineAt(i).text;
    const match = text.match(/^(\s*)with:\s*$/);
    if (match) {
      withIndent = match[1].length;
      withLine = i;
      break;
    }
  }
  if (withLine < 0) {
    return keys;
  }

  let entryIndent: number | undefined;
  for (let i = withLine + 1; i < document.lineCount; i++) {
    const text = document.lineAt(i).text;
    if (text.trim() === '') {
      continue;
    }
    const indent = text.match(/^(\s*)/)?.[1].length ?? 0;
    if (indent <= withIndent) {
      break;
    }
    if (entryIndent === undefined) {
      entryIndent = indent;
    }
    if (indent !== entryIndent) {
      // Nested content under a with value (e.g. folded block).
      continue;
    }
    const keyMatch = text.match(/^\s*([A-Za-z_]\w*)\s*:/);
    if (keyMatch) {
      keys.add(keyMatch[1]);
    }
  }
  return keys;
}

function parseArgNames(source: string): string[] {
  const names: string[] = [];
  const argsBlock = source.match(/\nargs:\s*\n((?:[ \t]+-[\s\S]*?)?)(?=\n\w|\n*$)/);
  const block = argsBlock?.[1] ?? source;
  const namePattern = /(?:^|\n)[ \t]*-[ \t]*(?:\{[ \t]*)?name:\s*['"]?([A-Za-z_]\w*)['"]?/g;
  let match: RegExpExecArray | null;
  while ((match = namePattern.exec(block)) !== null) {
    names.push(match[1]);
  }
  // Also support list-of-maps with indented name:
  const alt = /(?:^|\n)[ \t]+name:\s*['"]?([A-Za-z_]\w*)['"]?/g;
  while ((match = alt.exec(source)) !== null) {
    if (!names.includes(match[1])) {
      names.push(match[1]);
    }
  }
  return names;
}
