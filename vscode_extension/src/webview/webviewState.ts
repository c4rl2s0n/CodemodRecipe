import { RunnerTab } from '../constants';
import { RecipeSchema } from '../types';

export interface RecipeViewState {
  readonly recipes: readonly RecipeSchema[];
  readonly discoveryError?: string;
  readonly recipe?: RecipeSchema;
  readonly initialArgs: Record<string, string>;
  readonly activeTab: RunnerTab;
}
