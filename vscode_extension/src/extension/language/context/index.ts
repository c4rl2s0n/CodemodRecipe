/** Luau recipe document context helpers (brace-aware, no full parser). */

export type {
  MethodSpecContext,
  RecipeIdContext,
  TextDocument,
} from './types';

export {
  documentTopLevelId,
  documentTopLevelIdLine,
  matchIdReference,
  recipeIdCompletionContext,
} from './recipeId';

export { matchTemplatePath } from './templatePath';

export {
  collectTableKeys,
  hostMethodCompletionContext,
  matchHostMethod,
  methodSpecContext,
} from './hostMethod';

export {
  argDefKeyCompletionContext,
  argsFieldCompletionContext,
  collectSetRunArgKeys,
  findNearbyRunRecipeId,
  inRunArgsTable,
  parseRecipeArgNames,
} from './argsTable';

export {
  findSiblingQueryAssign,
  firstQueryAssignLine,
  readLuauLongString,
  unquoteLuauString,
} from './strings';

export { moduleKeyCompletionContext } from './moduleKeys';
