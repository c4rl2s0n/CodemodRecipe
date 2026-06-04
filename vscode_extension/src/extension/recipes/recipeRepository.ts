import { DartBridge } from '../host/dartBridge';
import type { RecipeSchema } from '../../shared';

export class RecipeRepository {
  private recipes: RecipeSchema[] = [];
  private lastError: string | undefined;
  private lastRefreshAt: number | undefined;

  constructor(private readonly bridge: DartBridge) {}

  getRecipes(): readonly RecipeSchema[] {
    return this.recipes;
  }

  getLastError(): string | undefined {
    return this.lastError;
  }

  hasRecipes(): boolean {
    return this.recipes.length > 0;
  }

  shouldRefresh(maxAgeMs: number): boolean {
    if (this.lastError) return true;
    if (!this.lastRefreshAt) return true;
    if (!this.hasRecipes()) return true;
    return Date.now() - this.lastRefreshAt > maxAgeMs;
  }

  async refresh(): Promise<void> {
    try {
      this.recipes = await this.bridge.list();
      this.lastError = undefined;
      this.lastRefreshAt = Date.now();
    } catch (err) {
      this.recipes = [];
      this.lastError = err instanceof Error ? err.message : String(err);
    }
  }
}
