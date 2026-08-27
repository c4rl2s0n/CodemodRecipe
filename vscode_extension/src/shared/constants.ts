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

/** Host-owned recipe-runner session phase (source of truth for in-flight ops). */
export const RUNNER_SESSION_PHASE = {
  idle: 'idle',
  pickingPath: 'pickingPath',
  previewing: 'previewing',
  reviewing: 'reviewing',
  applying: 'applying',
} as const;

export type RunnerSessionPhase =
  (typeof RUNNER_SESSION_PHASE)[keyof typeof RUNNER_SESSION_PHASE];

export const ARG_INPUT_KIND = {
  text: 'text',
  bool: 'bool',
  choice: 'choice',
  file: 'file',
  directory: 'directory',
} as const;

export type ArgInputKind = (typeof ARG_INPUT_KIND)[keyof typeof ARG_INPUT_KIND];

export const FILE_PREVIEW_KIND = {
edit: 'edit',
create: 'create',
other: 'other'};
export type FilePreviewKind = (typeof FILE_PREVIEW_KIND)[keyof typeof FILE_PREVIEW_KIND];