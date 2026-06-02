export const EXTENSION = {
  activityViewId: 'workbench.view.extension.codemodRecipe',
} as const;

export const COMMANDS = {
  refresh: 'codemodRecipe.refresh',
  runRecipe: 'codemodRecipe.runRecipe',
  runFromCursorContext: 'codemodRecipe.runFromCursorContext',
  configureHost: 'codemodRecipe.configureHost',
} as const;

export const VIEWS = {
  runner: 'codemodRecipe.runner',
} as const;

export const CONFIG = {
  section: 'codemodRecipe',
  hostEntrypoint: 'hostEntrypoint',
  dartPath: 'dartPath',
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

export const DEFAULT_HOST_CANDIDATES = [
  'tool/codemod_host.dart',
  'tool/codemods/codemod_host.dart',
  'bin/codemod_host.dart',
  'example/vscode_host_example/bin/codemod_host.dart',
] as const;

export const WEBVIEW_ASSETS = {
  html: ['media', 'recipeView.html'],
  css: ['media', 'recipeView.css'],
  script: ['media', 'recipeView.js'],
} as const;

export const RUNNER_TABS = {
  recipes: 'recipes',
  runner: 'runner',
} as const;

export type RunnerTab = (typeof RUNNER_TABS)[keyof typeof RUNNER_TABS];

export const WEBVIEW_TO_EXTENSION = {
  showRecipes: 'showRecipes',
  showRunner: 'showRunner',
  selectRecipe: 'selectRecipe',
  refreshRecipes: 'refreshRecipes',
  configureHost: 'configureHost',
  pickFile: 'pickFile',
  pickDirectory: 'pickDirectory',
  preview: 'preview',
  openDiff: 'openDiff',
  apply: 'apply',
} as const;

export const EXTENSION_TO_WEBVIEW = {
  filePicked: 'filePicked',
  previewResult: 'previewResult',
  applyResult: 'applyResult',
  error: 'error',
} as const;
