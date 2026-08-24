import { ARG_INPUT_KIND } from '../shared';
import type { ArgValue, ArgValues, RecipeArg, RecipeSchema } from '../shared';

const BUILTINS = new Set<string>(Object.values(ARG_INPUT_KIND));

export function itemKind(kind: RecipeArg['inputKind'] | undefined): string {
  if (Array.isArray(kind)) {
    return kind[0] ?? ARG_INPUT_KIND.text;
  }
  if (!kind || kind === 'symbol' || kind === 'dartType') {
    return ARG_INPUT_KIND.text;
  }
  if (kind === 'enum') {
    return ARG_INPUT_KIND.choice;
  }
  return kind;
}

export function isListKind(kind: RecipeArg['inputKind'] | undefined): boolean {
  return Array.isArray(kind);
}

export function isNamedType(kind: string): boolean {
  return !BUILTINS.has(kind);
}

export function formatInputKind(kind: RecipeArg['inputKind'] | undefined): string {
  if (isListKind(kind)) {
    return `list of ${itemKind(kind)}`;
  }
  return itemKind(kind);
}

export function effectiveInputKind(arg: RecipeArg): string {
  const kind = itemKind(arg.inputKind);
  if (kind !== ARG_INPUT_KIND.text) {
    return kind;
  }
  return looksLikePath(arg.name) ? ARG_INPUT_KIND.file : ARG_INPUT_KIND.text;
}

export function looksLikePath(name: string): boolean {
  return (
    name === 'file' ||
    name === 'path' ||
    name.endsWith('_file') ||
    name.endsWith('Path')
  );
}

export function emptyValue(arg: RecipeArg): ArgValue {
  if (isListKind(arg.inputKind)) {
    return [];
  }
  const kind = itemKind(arg.inputKind);
  if (kind === ARG_INPUT_KIND.bool) {
    return arg.defaultsTo === 'true';
  }
  if (isNamedType(kind) || (arg.fields && arg.fields.length > 0)) {
    const obj: Record<string, ArgValue> = {};
    for (const field of arg.fields ?? []) {
      obj[field.name] = emptyValue(field);
    }
    return obj;
  }
  return arg.defaultsTo ?? '';
}

export function collectArgs(
  recipe: RecipeSchema | undefined,
  argValues: ArgValues
): ArgValues {
  const args: ArgValues = {};
  if (!recipe) {
    return args;
  }
  for (const arg of recipe.args) {
    const value = argValues[arg.name];
    if (value === undefined || value === '') {
      continue;
    }
    args[arg.name] = value;
  }
  return args;
}

export function argsKey(args: ArgValues): string {
  return JSON.stringify(canonicalize(args));
}

function canonicalize(value: ArgValue): unknown {
  if (Array.isArray(value)) {
    return value.map(canonicalize);
  }
  if (value && typeof value === 'object') {
    const out: Record<string, unknown> = {};
    for (const key of Object.keys(value).sort()) {
      out[key] = canonicalize(value[key]);
    }
    return out;
  }
  return value;
}

function isMissingValue(arg: RecipeArg, value: ArgValue | undefined): boolean {
  if (value === undefined || value === null) {
    return true;
  }
  if (typeof value === 'string' && value === '' && !isListKind(arg.inputKind)) {
    return true;
  }
  return false;
}

export function collectMissingRequiredArgs(
  recipe: RecipeSchema | undefined,
  argValues: ArgValues
): string[] {
  if (!recipe) {
    return [];
  }
  return recipe.args
    .filter((arg) => arg.required && isMissingValue(arg, argValues[arg.name]))
    .map((arg) => arg.name);
}

export function initArgValues(
  recipe: RecipeSchema | undefined,
  initialArgs: ArgValues
): ArgValues {
  const values: ArgValues = { ...initialArgs };
  if (!recipe) {
    return values;
  }
  for (const arg of recipe.args) {
    if (values[arg.name] === undefined) {
      values[arg.name] = emptyValue(arg);
    }
  }
  return values;
}

/** Keeps user-entered values for args that still exist; drops removed args; fills new defaults. */
export function mergeArgValuesOnRefresh(
  recipe: RecipeSchema | undefined,
  existing: ArgValues
): ArgValues {
  if (!recipe) {
    return {};
  }
  const argNames = new Set(recipe.args.map((arg) => arg.name));
  const preserved: ArgValues = {};
  for (const [key, value] of Object.entries(existing)) {
    if (argNames.has(key)) {
      preserved[key] = value;
    }
  }
  return initArgValues(recipe, preserved);
}

export function rowSummary(fields: RecipeArg[] | undefined, row: ArgValue): string {
  if (!fields || typeof row !== 'object' || row === null || Array.isArray(row)) {
    return String(row ?? 'Item');
  }
  const parts = fields
    .filter((f) => f.required)
    .slice(0, 2)
    .map((f) => {
      const v = row[f.name];
      return typeof v === 'string' || typeof v === 'boolean' ? String(v) : '';
    })
    .filter(Boolean);
  return parts.join(' ') || 'Item';
}

export function asObject(value: ArgValue | undefined): Record<string, ArgValue> {
  if (value && typeof value === 'object' && !Array.isArray(value)) {
    return value;
  }
  return {};
}

export function asList(value: ArgValue | undefined): ArgValue[] {
  return Array.isArray(value) ? value : [];
}
