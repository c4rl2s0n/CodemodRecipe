import { DartBridge } from '../host/dartBridge';
import { RecipeSchema } from '../types';

export class RecipeRepository {
  private recipes: RecipeSchema[] = [];
  private lastError: string | undefined;

  constructor(private readonly bridge: DartBridge) {}

  getRecipes(): readonly RecipeSchema[] {
    return this.recipes;
  }

  getLastError(): string | undefined {
    return this.lastError;
  }

  async refresh(): Promise<void> {
    try {
      this.recipes = await this.bridge.list();
      this.lastError = undefined;
    } catch (err) {
      this.recipes = [];
      this.lastError = err instanceof Error ? err.message : String(err);
    }
  }
}
