import * as path from 'path';
import * as vscode from 'vscode';
import { COMMANDS } from '../constants';
import type { ExtensionConfig } from '../config/extensionConfig';
import type { HostBridge } from '../host/hostBridge';
import type { RecipeRepository } from '../recipes/recipeRepository';
import { loadDslSurface } from './dslSurface';
import { RecipeCodeLensProvider } from './providers/codelens';
import { RecipeCompletionProvider } from './providers/completion';
import { RecipeDefinitionProvider } from './providers/definition';
import { RecipeHoverProvider } from './providers/hover';

function yamlSelectorForCodemodRoot(codemodRoot: string): vscode.DocumentSelector {
  const normalized = codemodRoot.replace(/\\/g, '/').replace(/^\/+|\/+$/g, '');
  const pattern = `**/${normalized}/**/*.{yaml,yml}`;
  return [
    { language: 'yaml', pattern },
    { language: 'yaml', scheme: 'file', pattern: `**/${normalized}/**` },
  ];
}

let languageCommandsRegistered = false;

function registerLanguageCommandsOnce(
  context: vscode.ExtensionContext,
  repository: RecipeRepository,
  config: ExtensionConfig
): void {
  if (languageCommandsRegistered) {
    return;
  }
  languageCommandsRegistered = true;
  context.subscriptions.push(
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

/**
 * Owns language feature registration for the current workspace / codemod root.
 * Dispose and recreate when roots change (commands stay registered once).
 */
export class LanguageSession implements vscode.Disposable {
  private readonly disposables: vscode.Disposable[] = [];

  constructor(
    private readonly context: vscode.ExtensionContext,
    private readonly repository: RecipeRepository,
    private readonly bridge: HostBridge,
    private readonly config: ExtensionConfig
  ) {
    registerLanguageCommandsOnce(context, repository, config);
    this.registerProviders();
  }

  private isUnderCodemod = (uri: vscode.Uri): boolean => {
    const root = path.normalize(
      path.join(this.config.workspaceRoot, this.config.codemodRoot)
    );
    const file = path.normalize(uri.fsPath);
    return file === root || file.startsWith(root + path.sep);
  };

  private registerProviders(): void {
    const selector = yamlSelectorForCodemodRoot(this.config.codemodRoot);
    const surface = loadDslSurface(this.context.extensionUri);

    this.disposables.push(
      vscode.languages.registerDefinitionProvider(
        selector,
        new RecipeDefinitionProvider(
          this.repository,
          this.config,
          this.isUnderCodemod
        )
      ),
      vscode.languages.registerCompletionItemProvider(
        selector,
        new RecipeCompletionProvider(
          this.repository,
          this.bridge,
          surface,
          this.isUnderCodemod
        ),
        ':',
        ' ',
        '.',
        '{',
        '"'
      ),
      vscode.languages.registerHoverProvider(
        selector,
        new RecipeHoverProvider(
          this.context.extensionUri,
          this.repository,
          this.config,
          this.isUnderCodemod
        )
      ),
      vscode.languages.registerCodeLensProvider(
        selector,
        new RecipeCodeLensProvider(this.isUnderCodemod)
      )
    );
  }

  dispose(): void {
    for (const d of this.disposables) {
      d.dispose();
    }
    this.disposables.length = 0;
  }
}

export function registerRecipeLanguageSupport(
  context: vscode.ExtensionContext,
  repository: RecipeRepository,
  bridge: HostBridge,
  config: ExtensionConfig
): LanguageSession {
  return new LanguageSession(context, repository, bridge, config);
}
