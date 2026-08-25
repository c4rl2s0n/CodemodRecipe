/**
 * Pure completion suggestion logic for Luau recipes (no vscode dependency).
 */

import { collectRecipeIdCompletions } from '../../shared';
import type { RecipeArg, RecipeSchema, TypeSchema } from '../../shared';
import { collectCaptureNameCompletions } from './captureCompletions';
import {
  ARG_DEFINITION_KEYS,
  BUILTIN_INPUT_KINDS,
  HOST_CONTEXT_FIELD,
  HOST_METHODS,
  RECIPE_MODULE_KEYS,
} from './hostApiSurface';
import {
  argDefKeyCompletionContext,
  argsFieldCompletionContext,
  findNearbyRunRecipeId,
  hostMethodCompletionContext,
  inRunArgsTable,
  methodSpecContext,
  moduleKeyCompletionContext,
  parseRecipeArgNames,
  recipeIdCompletionContext,
  type TextDocument,
} from './luauContext';

export type CompletionKind =
  | 'method'
  | 'field'
  | 'property'
  | 'module'
  | 'reference'
  | 'value'
  | 'enumMember'
  | 'variable'
  | 'enum';

export type CompletionSuggestion = {
  label: string;
  kind: CompletionKind;
  detail?: string;
  insertText?: string;
  sortText?: string;
  /** Character range on the current line to replace. */
  range?: { start: number; end: number };
};

export type CompletionCatalog = {
  recipeIds: readonly string[];
  mapIds: readonly string[];
  varIds: readonly string[];
  queryIds: readonly string[];
  languageIds: readonly string[];
  typeIds: readonly string[];
  types: readonly TypeSchema[];
  getQuerySource: (id: string) => string | undefined;
  /** Optional: resolve nested recipe for c:run args. */
  describeRecipe?: (id: string) => RecipeSchema | undefined;
  readScmFile?: (relativePath: string) => string | undefined;
};

export type CompletionRequest = {
  line: string;
  before: string;
  lineNumber: number;
  character: number;
  document: TextDocument;
  catalog: CompletionCatalog;
};

function captureSuggestions(
  document: TextDocument,
  lineNumber: number,
  typed: string,
  catalog: CompletionCatalog
): CompletionSuggestion[] | undefined {
  const lineSource = {
    lineCount: document.lineCount,
    lineText: (i: number) => document.lineAt(i).text,
  };
  const names = collectCaptureNameCompletions(
    lineSource,
    lineNumber,
    typed,
    catalog.readScmFile,
    catalog.getQuerySource
  );
  if (names.length === 0) {
    return undefined;
  }
  return names.map((name) => ({
    label: name,
    kind: 'field' as const,
    detail: 'query capture',
    sortText: `0_${name}`,
  }));
}

/**
 * Collect completion suggestions for a Luau recipe cursor position.
 * Returns undefined when no recipe-specific suggestions apply.
 */
export function collectCompletionSuggestions(
  req: CompletionRequest
): CompletionSuggestion[] | undefined {
  const { before, line, lineNumber, character, document, catalog } = req;
  const text = document.getText();

  const argsCtx = argsFieldCompletionContext(before);
  if (argsCtx) {
    const names = parseRecipeArgNames(text);
    return names
      .filter((n) => !argsCtx.typed || n.startsWith(argsCtx.typed))
      .map((name) => ({
        label: name,
        kind: 'field' as const,
        detail: 'c.args',
        range: { start: argsCtx.start, end: character },
      }));
  }

  const methodCtx = hostMethodCompletionContext(before);
  if (methodCtx) {
    const items: CompletionSuggestion[] = HOST_METHODS.filter((m) =>
      m.name.startsWith(methodCtx.typed)
    ).map((m) => ({
      label: m.name,
      kind: 'method' as const,
      detail: m.description,
      range: { start: methodCtx.start, end: character },
    }));
    if (!methodCtx.typed || HOST_CONTEXT_FIELD.startsWith(methodCtx.typed)) {
      items.unshift({
        label: HOST_CONTEXT_FIELD,
        kind: 'field',
        detail: 'Resolved recipe arguments',
        range: { start: methodCtx.start, end: character },
      });
    }
    return items;
  }

  const idCtx = recipeIdCompletionContext(before);
  if (idCtx) {
    let ids: readonly string[] = [];
    let detailPrefix = 'Recipe id';
    if (idCtx.kind === 'map') {
      ids = catalog.mapIds;
      detailPrefix = 'Map id';
    } else if (idCtx.kind === 'var') {
      ids = catalog.varIds;
      detailPrefix = 'Var id';
    } else if (idCtx.kind === 'queryLib') {
      ids = catalog.queryIds;
      detailPrefix = 'Query id';
    } else {
      ids = catalog.recipeIds;
    }
    const segmentStart = idCtx.start + idCtx.typed.lastIndexOf('.') + 1;
    return collectRecipeIdCompletions(ids, idCtx.typed).map((completion) => ({
      label: completion.label,
      kind: completion.hasChildren ? ('module' as const) : ('reference' as const),
      insertText: completion.insertText,
      detail: completion.hasChildren
        ? `${detailPrefix} group: ${completion.fullPath}`
        : `${detailPrefix}: ${completion.fullPath}`,
      sortText: `1_${completion.label}`,
      range: { start: segmentStart, end: character },
    }));
  }

  const specCtx = methodSpecContext(document, lineNumber, character);
  if (specCtx) {
    if (specCtx.valueField) {
      const enumValues = specCtx.spec.enums?.[specCtx.valueField];
      if (enumValues) {
        const typed = specCtx.typedValue ?? '';
        return enumValues
          .filter((v) => !typed || v.startsWith(typed))
          .map((v) => ({
            label: v,
            kind: 'enumMember' as const,
            sortText: `0_${v}`,
            range:
              specCtx.valueStart !== undefined
                ? { start: specCtx.valueStart, end: character }
                : undefined,
          }));
      }
      if (specCtx.valueField === 'language') {
        const typed = specCtx.typedValue ?? '';
        return catalog.languageIds
          .filter((id) => !typed || id.startsWith(typed))
          .map((id) => ({
            label: id,
            kind: 'value' as const,
            detail: 'language',
          }));
      }
      if (specCtx.valueField === 'capture') {
        return captureSuggestions(
          document,
          lineNumber,
          specCtx.typedValue ?? '',
          catalog
        );
      }
      if (specCtx.valueField === 'query') {
        const typed = specCtx.typedValue ?? '';
        return catalog.queryIds
          .filter((id) => !typed || id.startsWith(typed))
          .map((id) => ({
            label: id,
            kind: 'reference' as const,
            detail: 'registered query',
          }));
      }
    } else {
      const keys = specCtx.spec.keys.filter(
        (k) =>
          !specCtx.setKeys.has(k) &&
          (!specCtx.typedKey || k.startsWith(specCtx.typedKey))
      );
      if (keys.length > 0) {
        return keys.map((key) => ({
          label: key,
          kind: 'property' as const,
          insertText: `${key} = `,
          detail: `c:${specCtx.method}`,
          sortText: `0_${key}`,
          range: { start: specCtx.keyStart, end: character },
        }));
      }
    }
  }

  const langMatch = /language\s*=\s*['"]([\w-]*)$/.exec(before);
  if (langMatch) {
    const typed = langMatch[1] ?? '';
    return catalog.languageIds
      .filter((id) => !typed || id.startsWith(typed))
      .map((id) => ({ label: id, kind: 'value' as const }));
  }

  const captureMatch = /capture\s*=\s*['"]([\w]*)$/.exec(before);
  if (captureMatch) {
    return captureSuggestions(document, lineNumber, captureMatch[1] ?? '', catalog);
  }

  const inputKindMatch = /inputKind\s*=\s*['"]([\w]*)$/.exec(before);
  if (inputKindMatch) {
    const typed = inputKindMatch[1] ?? '';
    const kinds = [...BUILTIN_INPUT_KINDS, ...catalog.typeIds];
    return kinds
      .filter((k) => !typed || k.startsWith(typed))
      .map((k) => ({
        label: k,
        kind: 'enumMember' as const,
        detail: (BUILTIN_INPUT_KINDS as readonly string[]).includes(k)
          ? 'builtin inputKind'
          : 'named type',
      }));
  }

  if (inRunArgsTable(document, lineNumber, character)) {
    const childId = findNearbyRunRecipeId(document, lineNumber);
    if (childId && catalog.describeRecipe) {
      const described = catalog.describeRecipe(childId);
      if (described) {
        const keyMatch = /^\s*([A-Za-z_]\w*)$/.exec(before);
        const typed = keyMatch?.[1] ?? '';
        const remaining = described.args
          .filter((arg) => !typed || arg.name.startsWith(typed))
          .sort((a, b) => {
            if (a.required !== b.required) {
              return a.required ? -1 : 1;
            }
            return a.name.localeCompare(b.name);
          });
        if (remaining.length > 0) {
          return remaining.map((arg: RecipeArg) => {
            const parts: string[] = [];
            if (arg.required) {
              parts.push('required');
            }
            if (arg.help) {
              parts.push(arg.help);
            }
            return {
              label: arg.name,
              kind: 'property' as const,
              detail: parts.length ? parts.join(' — ') : `c:run(${childId})`,
              insertText: `${arg.name} = `,
              sortText: `${arg.required ? '0' : '1'}_${arg.name}`,
            };
          });
        }
      }
    }
  }

  const argDefCtx = argDefKeyCompletionContext(document, lineNumber, before);
  if (argDefCtx) {
    const argKeys: CompletionSuggestion[] = ARG_DEFINITION_KEYS.filter(
      (k) => !argDefCtx.typed || k.startsWith(argDefCtx.typed)
    ).map((key) => ({
      label: key,
      kind: 'property' as const,
      insertText: `${key} = `,
      detail: 'arg definition',
      range: { start: argDefCtx.start, end: character },
      sortText: `0_${key}`,
    }));
    const seen = new Set(argKeys.map((i) => i.label));
    for (const type of catalog.types) {
      for (const field of type.fields) {
        if (seen.has(field.name)) {
          continue;
        }
        if (argDefCtx.typed && !field.name.startsWith(argDefCtx.typed)) {
          continue;
        }
        seen.add(field.name);
        argKeys.push({
          label: field.name,
          kind: 'field',
          detail: `type ${type.id}`,
          insertText: `${field.name} = `,
          range: { start: argDefCtx.start, end: character },
          sortText: `1_${type.id}_${field.name}`,
        });
      }
    }
    if (argKeys.length > 0) {
      return argKeys;
    }
  }

  const modCtx = moduleKeyCompletionContext(before, line);
  if (modCtx) {
    return RECIPE_MODULE_KEYS.filter(
      (k) => !modCtx.typed || k.startsWith(modCtx.typed)
    ).map((key) => ({
      label: key,
      kind: 'property' as const,
      insertText: `${key} = `,
      detail: 'recipe module',
      range: { start: modCtx.start, end: character },
    }));
  }

  if (/\{\{\s*[\w.]*$/.test(before)) {
    const args = parseRecipeArgNames(text);
    const items: CompletionSuggestion[] = args.map((name) => ({
      label: name,
      kind: 'field' as const,
      detail: 'recipe arg',
    }));
    for (const id of catalog.varIds) {
      items.push({
        label: `var.${id}`,
        kind: 'variable',
        insertText: `var.${id}`,
      });
    }
    for (const id of catalog.mapIds) {
      items.push({
        label: `map.${id}`,
        kind: 'enum',
        insertText: `map.${id}|`,
        detail: 'map filter (append key)',
      });
    }
    return items;
  }

  return undefined;
}
