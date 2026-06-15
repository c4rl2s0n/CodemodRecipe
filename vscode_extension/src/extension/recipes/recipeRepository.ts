import { DartBridge } from '../host/dartBridge';
import type { RecipeDiagnostic, RecipeSchema } from '../../shared';

export type RecipeLoadResult = {
  recipes: RecipeSchema[];
  diagnostics: RecipeDiagnostic[];
};

export class RecipeRepository {
  private recipes: RecipeSchema[] = [];
  private diagnostics: RecipeDiagnostic[] = [];
  private lastError: string | undefined;

  constructor(private readonly bridge: DartBridge) {}

  getRecipes(): readonly RecipeSchema[] {
    return this.recipes;
  }

  getDiagnostics(): readonly RecipeDiagnostic[] {
    return this.diagnostics;
  }

  getLastError(): string | undefined {
    return this.lastError;
  }

  async refresh(): Promise<void> {
    try {
      const result = await this.bridge.listRecipes();
      this.applyLoadResult(result);
      this.lastError = undefined;
    } catch (err) {
      this.recipes = [];
      this.diagnostics = [];
      this.lastError = err instanceof Error ? err.message : String(err);
    }
  }

  async reload(): Promise<void> {
    try {
      const result = await this.bridge.reloadRecipes();
      this.applyLoadResult(result);
      this.lastError = undefined;
    } catch (err) {
      this.recipes = [];
      this.diagnostics = [];
      this.lastError = err instanceof Error ? err.message : String(err);
    }
  }

  private applyLoadResult(result: RecipeLoadResult): void {
    this.recipes = result.recipes;
    this.diagnostics = result.diagnostics;
  }
}
