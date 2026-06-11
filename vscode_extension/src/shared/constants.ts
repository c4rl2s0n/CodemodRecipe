export const RUNNER_TABS = {
  recipes: 'recipes',
  runner: 'runner',
} as const;

export type RunnerTab = (typeof RUNNER_TABS)[keyof typeof RUNNER_TABS];

export const BOOTSTRAP_PHASES = {
  startingHost: 'startingHost',
  loadingRecipes: 'loadingRecipes',
  ready: 'ready',
  error: 'error',
} as const;

export type BootstrapPhase =
  (typeof BOOTSTRAP_PHASES)[keyof typeof BOOTSTRAP_PHASES];

export const ARG_INPUT_KIND = {
  text: 'text',
  file: 'file',
  directory: 'directory',
  enum: 'enum',
  dartType: 'dartType',
  symbol: 'symbol',
} as const;

export type ArgInputKind = (typeof ARG_INPUT_KIND)[keyof typeof ARG_INPUT_KIND];

export const FILE_PREVIEW_KIND = {
edit: 'edit',
create: 'create',
other: 'other'};
export type FilePreviewKind = (typeof FILE_PREVIEW_KIND)[keyof typeof FILE_PREVIEW_KIND];