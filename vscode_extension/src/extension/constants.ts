export const EXTENSION = {
  activityViewId: 'workbench.view.extension.codemodRecipe',
} as const;

export const COMMANDS = {
  refresh: 'codemodRecipe.refresh',
  bootstrap: 'codemodRecipe.bootstrap',
  scaffoldProject: 'codemodRecipe.scaffoldProject',
  runRecipe: 'codemodRecipe.runRecipe',
  runFromCursorContext: 'codemodRecipe.runFromCursorContext',
  configureCodemodRoot: 'codemodRecipe.configureHost',
  validateRecipes: 'codemodRecipe.validateRecipes',
  openInRecipeRunner: 'codemodRecipe.openInRecipeRunner',
  testQueryOnFile: 'codemodRecipe.testQueryOnFile',
} as const;

export const VIEWS = {
  runner: 'codemodRecipe.runner',
} as const;

export const CONFIG = {
  section: 'codemodRecipe',
  workspaceRoot: 'workspaceRoot',
  codemodRoot: 'codemodRoot',
  performanceLogging: 'performanceLogging',
  autoPreviewDebounceMs: 'autoPreviewDebounceMs',
  previewSnippetLines: 'previewSnippetLines',
} as const;

export const DIFF = {
  scheme: 'codemod-diff',
  originalPrefix: '/original',
  modifiedPrefix: '/modified',
} as const;

export const HOST_PROTOCOL = {
  resultBegin: '__CODEMOD_RESULT_BEGIN__',
  resultEnd: '__CODEMOD_RESULT_END__',
} as const;

export const WEBVIEW_ASSETS = {
  html: ['media', 'recipeView.html'],
  css: ['media', 'recipeView.css'],
  script: ['media', 'recipeView.js'],
} as const;
