export const EXTENSION = {
  activityViewId: 'workbench.view.extension.codemodRecipe',
} as const;

export const COMMANDS = {
  refresh: 'codemodRecipe.refresh',
  bootstrap: 'codemodRecipe.bootstrap',
  scaffoldProject: 'codemodRecipe.scaffoldProject',
  runRecipe: 'codemodRecipe.runRecipe',
  runFromCursorContext: 'codemodRecipe.runFromCursorContext',
  runFromExplorer: 'codemodRecipe.runFromExplorer',
  openFromExplorer: 'codemodRecipe.openFromExplorer',
  invoke: 'codemodRecipe.invoke',
  invokeSlot: 'codemodRecipe.invokeSlot',
  copyInvokeKeybinding: 'codemodRecipe.copyInvokeKeybinding',
  assignToSlot: 'codemodRecipe.assignToSlot',
  copySlotKeybinding: 'codemodRecipe.copySlotKeybinding',
  configureCodemodRoot: 'codemodRecipe.configureHost',
  validateRecipes: 'codemodRecipe.validateRecipes',
  openInRecipeRunner: 'codemodRecipe.openInRecipeRunner',
  testQueryOnFile: 'codemodRecipe.testQueryOnFile',
  queryToolsRun: 'codemodRecipe.queryTools.run',
  queryToolsGenerateFromCursor: 'codemodRecipe.queryTools.generateFromCursor',
  queryToolsRevealAst: 'codemodRecipe.queryTools.revealAst',
  queryToolsNextMatch: 'codemodRecipe.queryTools.nextMatch',
  queryToolsPrevMatch: 'codemodRecipe.queryTools.prevMatch',
  queryToolsCopy: 'codemodRecipe.queryTools.copy',
  queryToolsCopyYamlInsert: 'codemodRecipe.queryTools.copyYaml.insert',
  queryToolsCopyYamlReplace: 'codemodRecipe.queryTools.copyYaml.replace',
  queryToolsCopyYamlRemove: 'codemodRecipe.queryTools.copyYaml.remove',
  queryToolsOpenFromRecipe: 'codemodRecipe.queryTools.openFromRecipe',
  queryToolsGoToEditPath: 'codemodRecipe.queryTools.goToEditPath',
} as const;

export const VIEWS = {
  runner: 'codemodRecipe.runner',
  queryAst: 'codemodRecipe.queryAst',
  queryEditor: 'codemodRecipe.queryEditor',
} as const;

export const CONFIG = {
  section: 'codemodRecipe',
  workspaceRoot: 'workspaceRoot',
  codemodRoot: 'codemodRoot',
  performanceLogging: 'performanceLogging',
  autoPreviewDebounceMs: 'autoPreviewDebounceMs',
  previewSnippetLines: 'previewSnippetLines',
  slots: 'slots',
  shortcutConfirmApply: 'shortcutConfirmApply',
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
