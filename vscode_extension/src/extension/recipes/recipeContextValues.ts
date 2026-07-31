import * as path from 'path';
import type { ArgFrom, RecipeArg, RecipeSchema } from '../../shared';

/** Resolve string `from` / `contextKey` and template `from` against builtin context. */
export function prefillArgs(
  recipe: RecipeSchema,
  contextValues: Record<string, string>
): Record<string, string> {
  const args: Record<string, string> = {};
  for (const arg of recipe.args) {
    const value = resolveLocalFrom(arg, contextValues);
    if (value) {
      args[arg.name] = value;
    }
  }
  return args;
}

export function argNeedsHostDerive(arg: RecipeArg): boolean {
  const from = effectiveFrom(arg);
  return typeof from === 'object' && from !== null && Boolean(from.query);
}

export function recipeNeedsHostDerive(recipe: RecipeSchema): boolean {
  return recipe.args.some(argNeedsHostDerive);
}

export function effectiveFrom(arg: RecipeArg): ArgFrom | null {
  if (arg.from != null) {
    return arg.from;
  }
  if (arg.contextKey) {
    return arg.contextKey;
  }
  return null;
}

function resolveLocalFrom(
  arg: RecipeArg,
  contextValues: Record<string, string>
): string | undefined {
  const from = effectiveFrom(arg);
  if (from == null) {
    return undefined;
  }
  if (typeof from === 'string') {
    const value = contextValues[from];
    return value ? value : undefined;
  }
  if (from.query) {
    // Host deriveArgs handles query forms.
    return undefined;
  }
  if (from.template) {
    const rendered = renderContextTemplate(from.template, contextValues);
    return rendered ? rendered : undefined;
  }
  return undefined;
}

/**
 * Minimal template renderer for shortcut builtins:
 * `{{ name }}` and `{{ name | filter }}` with filters basename, dirname, stem.
 */
export function renderContextTemplate(
  template: string,
  values: Record<string, string>
): string {
  return template.replace(/\{\{\s*([^}]+?)\s*\}\}/g, (_, expr: string) => {
    const parts = expr.split('|').map((p) => p.trim());
    let value = values[parts[0]] ?? '';
    for (let i = 1; i < parts.length; i++) {
      value = applyFilter(value, parts[i]);
    }
    return value;
  });
}

function applyFilter(value: string, filter: string): string {
  const name = filter.replace(/\(.*\)$/, '').trim();
  switch (name) {
    case 'basename':
      return path.basename(value);
    case 'dirname':
      return path.dirname(value).replace(/\\/g, '/');
    case 'stem': {
      const base = path.basename(value);
      const ext = path.extname(base);
      return ext ? base.slice(0, -ext.length) : base;
    }
    default:
      return value;
  }
}

export function missingRequiredArgNames(
  recipe: RecipeSchema,
  argValues: Record<string, string>
): string[] {
  const withDefaults: Record<string, string> = { ...argValues };
  for (const arg of recipe.args) {
    if (
      (withDefaults[arg.name] === undefined || withDefaults[arg.name] === '') &&
      arg.defaultsTo
    ) {
      withDefaults[arg.name] = arg.defaultsTo;
    }
  }
  return recipe.args
    .filter((arg) => arg.required && !withDefaults[arg.name])
    .map((arg) => arg.name);
}

export function mergeArgLayers(
  derived: Record<string, string>,
  overrides: Record<string, string>
): Record<string, string> {
  return { ...derived, ...overrides };
}
