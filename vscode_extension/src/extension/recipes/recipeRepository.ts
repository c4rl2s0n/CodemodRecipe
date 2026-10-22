import { HostBridge } from '../host/hostBridge';
import type {
  ComponentDetailSchema,
  ComponentSchema,
  RecipeDiagnostic,
  RecipeSchema,
  StackSchema,
  TestSuiteSchema,
  TypeSchema,
} from '../../shared';

export type RecipeLoadResult = {
  recipes: RecipeSchema[];
  diagnostics: RecipeDiagnostic[];
  mapIds?: string[];
  varIds?: string[];
  languageIds?: string[];
  typeIds?: string[];
  types?: TypeSchema[];
  queryIds?: string[];
  /** Registered query id → source string. */
  queries?: Record<string, string>;
  components?: ComponentSchema[];
  stack?: StackSchema;
  testSuites?: TestSuiteSchema[];
};

export class RecipeRepository {
  private recipes: RecipeSchema[] = [];
  private diagnostics: RecipeDiagnostic[] = [];
  private mapIds: string[] = [];
  private varIds: string[] = [];
  private languageIds: string[] = [];
  private typeIds: string[] = [];
  private types: TypeSchema[] = [];
  private queryIds: string[] = [];
  private queries: Record<string, string> = {};
  private components: ComponentSchema[] = [];
  private stack: StackSchema | undefined;
  private testSuites: TestSuiteSchema[] = [];
  private describeCache = new Map<string, RecipeSchema>();
  private componentDescribeCache = new Map<string, ComponentDetailSchema>();
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

  getTypeIds(): readonly string[] {
    return this.typeIds;
  }

  getTypes(): readonly TypeSchema[] {
    return this.types;
  }

  getQueryIds(): readonly string[] {
    return this.queryIds;
  }

  getQuerySource(id: string): string | undefined {
    return this.queries[id];
  }

  getComponents(): readonly ComponentSchema[] {
    return this.components;
  }

  getStack(): StackSchema | undefined {
    return this.stack;
  }

  getTestSuites(): readonly TestSuiteSchema[] {
    return this.testSuites;
  }

  findType(id: string): TypeSchema | undefined {
    return this.types.find((t) => t.id === id);
  }

  getLastError(): string | undefined {
    return this.lastError;
  }

  findById(id: string): RecipeSchema | undefined {
    return this.recipes.find((recipe) => recipe.id === id);
  }

  findComponentById(id: string): ComponentSchema | undefined {
    return this.components.find((c) => c.id === id);
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

  async describeComponentCached(
    componentId: string
  ): Promise<ComponentDetailSchema | undefined> {
    const cached = this.componentDescribeCache.get(componentId);
    if (cached) {
      return cached;
    }
    const described = await this.bridge.describeComponent(componentId);
    this.componentDescribeCache.set(componentId, described);
    return described;
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
      this.lastError = err instanceof Error ? err.message : String(err);
      throw err;
    }
  }

  private applyLoadResult(result: RecipeLoadResult): void {
    this.recipes = result.recipes;
    this.diagnostics = result.diagnostics;
    this.mapIds = result.mapIds ?? [];
    this.varIds = result.varIds ?? [];
    this.languageIds = result.languageIds ?? [];
    this.typeIds = result.typeIds ?? [];
    this.types = result.types ?? [];
    this.queryIds = result.queryIds ?? [];
    this.queries = result.queries ?? {};
    this.components = result.components ?? [];
    this.stack = result.stack;
    this.testSuites = result.testSuites ?? [];
    this.describeCache.clear();
    this.componentDescribeCache.clear();
  }
}
