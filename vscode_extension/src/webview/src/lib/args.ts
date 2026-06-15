import type { RecipeArg, RecipeSchema } from '../shared';

export function effectiveInputKind(arg: RecipeArg): string {
  if (arg.inputKind && arg.inputKind !== 'text') {
    return arg.inputKind;
  }
  return looksLikePath(arg.name) ? 'file' : 'text';
}

export function looksLikePath(name: string): boolean {
  return (
    name === 'file' ||
    name === 'path' ||
    name.endsWith('_file') ||
    name.endsWith('Path')
  );
}

export function collectArgs(
  recipe: RecipeSchema | undefined,
  argValues: Record<string, string>
): Record<string, string> {
  const args: Record<string, string> = {};
  if (!recipe) {
    return args;
  }
  for (const arg of recipe.args) {
    const value = argValues[arg.name];
    if (value) {
      args[arg.name] = value;
    }
  }
  return args;
}

export function argsKey(args: Record<string, string>): string {
  const ordered: Record<string, string> = {};
  for (const key of Object.keys(args).sort()) {
    ordered[key] = args[key];
  }
  return JSON.stringify(ordered);
}

export function collectMissingRequiredArgs(
  recipe: RecipeSchema | undefined,
  argValues: Record<string, string>
): string[] {
  if (!recipe) {
    return [];
  }
  const args = collectArgs(recipe, argValues);
  return recipe.args
    .filter((arg) => arg.required && !args[arg.name])
    .map((arg) => arg.name);
}

export function initArgValues(
  recipe: RecipeSchema | undefined,
  initialArgs: Record<string, string>
): Record<string, string> {
  const values: Record<string, string> = { ...initialArgs };
  if (!recipe) {
    return values;
  }
  for (const arg of recipe.args) {
    if (values[arg.name] === undefined) {
      values[arg.name] = arg.defaultsTo ?? '';
    }
  }
  return values;
}

/** Keeps user-entered values for args that still exist; drops removed args; fills new defaults. */
export function mergeArgValuesOnRefresh(
  recipe: RecipeSchema | undefined,
  existing: Record<string, string>
): Record<string, string> {
  if (!recipe) {
    return {};
  }
  const argNames = new Set(recipe.args.map((arg) => arg.name));
  const preserved: Record<string, string> = {};
  for (const [key, value] of Object.entries(existing)) {
    if (argNames.has(key)) {
      preserved[key] = value;
    }
  }
  return initArgValues(recipe, preserved);
}
