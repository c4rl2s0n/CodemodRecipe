/**
 * Pure navigation helpers for Luau recipes (definition / hover / CodeLens).
 */

import * as fs from 'fs';
import * as path from 'path';
import type { RecipeSchema } from '../../shared';
import {
  ARG_DEFINITION_DOCS,
  HOST_METHOD_DOCS,
  RECIPE_MODULE_DOCS,
} from './hostApiSurface';
import {
  documentTopLevelId,
  documentTopLevelIdLine,
  firstQueryAssignLine,
  matchHostMethod,
  matchIdReference,
  matchTemplatePath,
  type TextDocument,
} from './luauContext';

/**
 * Resolve template paths recipe-local first, then under codemod root
 * (mirrors host resource_path policy for go-to).
 */
export function resolveTemplatePath(
  workspaceRoot: string,
  codemodRoot: string,
  templateFile: string,
  recipeSourceFile?: string
): string | undefined {
  const candidates: string[] = [];
  if (recipeSourceFile) {
    const recipeDir = path.dirname(
      path.isAbsolute(recipeSourceFile)
        ? recipeSourceFile
        : path.join(workspaceRoot, recipeSourceFile)
    );
    candidates.push(path.join(recipeDir, templateFile));
  }
  candidates.push(path.join(workspaceRoot, codemodRoot, templateFile));
  for (const abs of candidates) {
    if (fs.existsSync(abs)) {
      return abs;
    }
  }
  return undefined;
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

export type DefinitionTarget =
  | { kind: 'recipe'; id: string; sourceFile: string }
  | { kind: 'template'; path: string };

export function resolveDefinitionTarget(
  line: string,
  character: number,
  _documentText: string,
  findRecipe: (id: string) => RecipeSchema | undefined
): DefinitionTarget | undefined {
  const ref = matchIdReference(line, character);
  if (ref && (ref.kind === 'run' || ref.kind === 'id')) {
    const target = findRecipe(ref.id);
    if (target?.sourceFile) {
      return { kind: 'recipe', id: ref.id, sourceFile: target.sourceFile };
    }
  }

  const templateFile = matchTemplatePath(line, character);
  if (templateFile) {
    return { kind: 'template', path: templateFile };
  }

  return undefined;
}

export type HoverContent =
  | { kind: 'markdown'; text: string }
  | { kind: 'code'; language: string; text: string; preface?: string };

export function resolveHoverContent(
  line: string,
  character: number,
  word: string | undefined,
  findRecipe: (id: string) => RecipeSchema | undefined,
  getQuerySource: (id: string) => string | undefined
): HoverContent | undefined {
  const method = matchHostMethod(line, character);
  if (method && HOST_METHOD_DOCS[method]) {
    return {
      kind: 'markdown',
      text: `**c:${method}**\n\n${HOST_METHOD_DOCS[method]}`,
    };
  }

  if (word && RECIPE_MODULE_DOCS[word]) {
    return {
      kind: 'markdown',
      text: `**${word}**\n\n${RECIPE_MODULE_DOCS[word]}`,
    };
  }
  if (word && ARG_DEFINITION_DOCS[word]) {
    return {
      kind: 'markdown',
      text: `**${word}** (arg)\n\n${ARG_DEFINITION_DOCS[word]}`,
    };
  }

  const recipeRef = matchIdReference(line, character);
  if (recipeRef && (recipeRef.kind === 'run' || recipeRef.kind === 'id')) {
    const described = findRecipe(recipeRef.id);
    if (!described) {
      return undefined;
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
    return {
      kind: 'markdown',
      text: `**${described.name}** (\`${described.id}\`)\n\n${described.description}\n\n**Args**\n${args}`,
    };
  }
  if (recipeRef?.kind === 'map') {
    return { kind: 'markdown', text: `**Map** \`${recipeRef.id}\`` };
  }
  if (recipeRef?.kind === 'var') {
    return { kind: 'markdown', text: `**Variable set** \`${recipeRef.id}\`` };
  }
  if (recipeRef?.kind === 'queryLib') {
    const src = getQuerySource(recipeRef.id);
    if (src) {
      return {
        kind: 'code',
        language: 'scheme',
        text: src.slice(0, 500),
        preface: `**Query** \`${recipeRef.id}\`\n\n`,
      };
    }
    return { kind: 'markdown', text: `**Query id** \`${recipeRef.id}\`` };
  }

  return undefined;
}

export type CodeLensDescriptor = {
  line: number;
  title: string;
  command: string;
  arguments: unknown[];
};

export function collectCodeLensDescriptors(
  document: TextDocument,
  commands: {
    openInRecipeRunner: string;
    copyInvokeKeybinding: string;
    assignToSlot: string;
    queryToolsOpenFromRecipe: string;
  }
): CodeLensDescriptor[] {
  const recipeId = documentTopLevelId(document);
  if (!recipeId) {
    return [];
  }
  const lenses: CodeLensDescriptor[] = [];
  const idLine = documentTopLevelIdLine(document);
  if (idLine !== undefined) {
    lenses.push(
      {
        line: idLine,
        title: 'Open in Recipe Runner',
        command: commands.openInRecipeRunner,
        arguments: [recipeId],
      },
      {
        line: idLine,
        title: 'Copy invoke keybinding',
        command: commands.copyInvokeKeybinding,
        arguments: [recipeId],
      },
      {
        line: idLine,
        title: 'Assign to slot…',
        command: commands.assignToSlot,
        arguments: [recipeId],
      }
    );
  }
  const queryLine = firstQueryAssignLine(document);
  if (queryLine !== undefined) {
    lenses.push({
      line: queryLine,
      title: 'Open in Query Tools',
      command: commands.queryToolsOpenFromRecipe,
      arguments: [queryLine],
    });
  }
  return lenses;
}
