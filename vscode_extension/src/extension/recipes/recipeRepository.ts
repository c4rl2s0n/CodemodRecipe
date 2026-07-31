import { HostBridge } from '../host/hostBridge';
import type { RecipeDiagnostic, RecipeSchema } from '../../shared';

export type RecipeLoadResult = {
  recipes: RecipeSchema[];
  diagnostics: RecipeDiagnostic[];
  mapIds?: string[];
  varIds?: string[];
  languageIds?: string[];
};

export class RecipeRepository {
  private recipes: RecipeSchema[] = [];
  private diagnostics: RecipeDiagnostic[] = [];
  private mapIds: string[] = [];
  private varIds: string[] = [];
  private languageIds: string[] = [];
  private describeCache = new Map<string, RecipeSchema>();
  private lastError: string | undefined;

  constructor(private readonly bridge: HostBridge) {}

  getRecipes(): readonly RecipeSchema[] {
    return this.recipes;
  }

  getDiagnostics(): readonly RecipeDiagnostic[] {
    return this.diagnostics;
  }

  getMapIds(): readonly string[] {
    return this.mapIds;
  }

  getVarIds(): readonly string[] {
    return this.varIds;
  }

  getLanguageIds(): readonly string[] {
    return this.languageIds;
  }

  getLastError(): string | undefined {
    return this.lastError;
  }

  findById(id: string): RecipeSchema | undefined {
    return this.recipes.find((recipe) => recipe.id === id);
  }

  async describeCached(recipeId: string): Promise<RecipeSchema | undefined> {
    const cached = this.describeCache.get(recipeId);
    if (cached) {
      return cached;
    }
    try {
      const described = await this.bridge.describe(recipeId);
      this.describeCache.set(recipeId, described);
      return described;
    } catch {
      return this.findById(recipeId);
    }
  }

  async refresh(): Promise<void> {
    try {
      const result = await this.bridge.listRecipes();
      this.applyLoadResult(result);
      this.lastError = undefined;
    } catch (err) {
      // Keep last-good catalog/diagnostics on transient host failure.
      this.lastError = err instanceof Error ? err.message : String(err);
    }
  }

  async reload(): Promise<void> {
    try {
      const result = await this.bridge.reloadRecipes();
      this.applyLoadResult(result);
      this.lastError = undefined;
    } catch (err) {
      // Keep last-good catalog/diagnostics on transient host failure.
      this.lastError = err instanceof Error ? err.message : String(err);
    }
  }

  private applyLoadResult(result: RecipeLoadResult): void {
    this.recipes = result.recipes;
    this.diagnostics = result.diagnostics;
    this.mapIds = result.mapIds ?? [];
    this.varIds = result.varIds ?? [];
    this.languageIds = result.languageIds ?? [];
    this.describeCache.clear();
  }

  private clearCatalog(): void {
    this.recipes = [];
    this.diagnostics = [];
    this.mapIds = [];
    this.varIds = [];
    this.languageIds = [];
    this.describeCache.clear();
  }

  /** Explicit wipe (e.g. tests); normal host errors keep last-good data. */
  resetForTests(): void {
    this.clearCatalog();
    this.lastError = undefined;
  }
}
