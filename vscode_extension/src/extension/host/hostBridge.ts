import type {
  ApplyResponse,
  DescribeResponse,
  DiffResponse,
  PreviewResponse,
  RecipeCatalogResponse,
  ValidateResponse,
  DumpAstResponse,
  DebugQueryResponse,
  GenerateQueryResponse,
  ResolveStaticPathResponse,
  QueryCapturesResponse,
} from './hostProtocol';
import type {
  ArgValues,
  RecipeSchema,
  SelectionPayload,
  BootstrapResponse,
  DeriveArgsRequest,
  DeriveArgsResponse,
  FilterExplorerRecipesResponse,
  ComponentScanResponse,
  ComponentCoverageResponse,
  ComponentInvokeResponse,
  DescribeComponentResponse,
  ComponentDetailSchema,
  TestRunReport,
} from '../../shared';
import type { RecipeLoadResult } from '../recipes/recipeRepository';
import { HostBridgeCore } from './hostBridgeCore';

export { shouldRetryOneShotAfterPersistentFailure } from './hostBridgeRetryPolicy';

export class HostBridge extends HostBridgeCore {
  async listRecipes(): Promise<RecipeLoadResult> {
    const response = await this.send<RecipeCatalogResponse>({ command: 'list' });
    return this.parseRecipeLoadResponse(response, 'list');
  }

  async reloadRecipes(): Promise<RecipeLoadResult> {
    const response = await this.send<RecipeCatalogResponse>({ command: 'reload' });
    return this.parseRecipeLoadResponse(response, 'reload');
  }

  async validateRecipes(recipeId?: string): Promise<ValidateResponse> {
    return this.send<ValidateResponse>({
      command: 'validate',
      ...(recipeId ? { recipe: recipeId } : {}),
    });
  }

  async ensureHost(): Promise<void> {
    await this.ensurePersistentHost();
  }

  async describe(recipe: string): Promise<RecipeSchema> {
    const response = await this.send<DescribeResponse>({
      command: 'describe',
      recipe,
    });
    if (!response.ok || !response.recipe) {
      throw new Error(response.error ?? `Failed to describe recipe: ${recipe}`);
    }
    return response.recipe;
  }

  preview(
    recipe: string,
    args: ArgValues,
    snippetLines?: number
  ): Promise<PreviewResponse> {
    return this.send<PreviewResponse>({
      command: 'preview',
      recipe,
      args,
      snippetLines,
    });
  }

  async diff(
    recipe: string,
    args: ArgValues,
    path: string
  ): Promise<DiffResponse> {
    return this.send<DiffResponse>({ command: 'diff', recipe, args, path });
  }

  apply(
    recipe: string,
    args: ArgValues,
    selection: SelectionPayload,
    previewToken: string
  ): Promise<ApplyResponse> {
    return this.send<ApplyResponse>({
      command: 'apply',
      recipe,
      args,
      selection,
      previewToken,
    });
  }

  deriveArgs(request: DeriveArgsRequest): Promise<DeriveArgsResponse> {
    return this.send<DeriveArgsResponse>({
      command: 'deriveArgs',
      recipe: request.recipe,
      source: request.source,
      language: request.language,
      path: request.path,
      cursorOffset: request.cursorOffset,
      selectionStart: request.selectionStart,
      selectionEnd: request.selectionEnd,
      context: request.context,
    });
  }

  filterExplorerRecipes(
    path: string,
    kind: 'file' | 'folder'
  ): Promise<FilterExplorerRecipesResponse> {
    return this.send<FilterExplorerRecipesResponse>({
      command: 'filterExplorerRecipes',
      path,
      kind,
    });
  }

  async bootstrapProject(
    force = false,
    options?: { editPolicy?: string; companions?: string[] }
  ): Promise<BootstrapResponse> {
    return this.send<BootstrapResponse>({
      command: 'bootstrap',
      force,
      ...(options?.editPolicy ? { editPolicy: options.editPolicy } : {}),
      ...(options?.companions ? { companions: options.companions } : {}),
    });
  }

  dumpAst(opts: {
    path?: string;
    source?: string;
    language?: string;
    namedOnly?: boolean;
  }): Promise<DumpAstResponse> {
    return this.send<DumpAstResponse>({
      command: 'dumpAst',
      path: opts.path,
      source: opts.source,
      language: opts.language,
      namedOnly: opts.namedOnly ?? true,
    });
  }

  debugQuery(opts: {
    path?: string;
    source?: string;
    language?: string;
    query: string;
    instrument?: boolean;
    includeSexp?: boolean;
  }): Promise<DebugQueryResponse> {
    return this.send<DebugQueryResponse>({
      command: 'debugQuery',
      path: opts.path,
      source: opts.source,
      language: opts.language,
      query: opts.query,
      instrument: opts.instrument ?? true,
      includeSexp: opts.includeSexp ?? false,
    });
  }

  generateQuery(opts: {
    path?: string;
    source?: string;
    language?: string;
    start: number;
    end?: number;
    includeTextPredicates?: boolean;
    captureLeaf?: string;
    maxDepth?: number;
  }): Promise<GenerateQueryResponse> {
    return this.send<GenerateQueryResponse>({
      command: 'generateQuery',
      path: opts.path,
      source: opts.source,
      language: opts.language,
      start: opts.start,
      end: opts.end ?? opts.start,
      includeTextPredicates: opts.includeTextPredicates ?? false,
      captureLeaf: opts.captureLeaf,
      maxDepth: opts.maxDepth,
    });
  }

  resolveStaticPath(template: string): Promise<ResolveStaticPathResponse> {
    return this.send<ResolveStaticPathResponse>({
      command: 'resolveStaticPath',
      template,
    });
  }

  queryCaptures(opts: {
    query?: string;
    queryId?: string;
  }): Promise<QueryCapturesResponse> {
    return this.send<QueryCapturesResponse>({
      command: 'queryCaptures',
      query: opts.query,
      queryId: opts.queryId,
    });
  }

  async componentScan(): Promise<ComponentScanResponse> {
    return this.send<ComponentScanResponse>({ command: 'componentScan' });
  }

  /**
   * Run `*.test.luau` suites. `filter` matches a suite id, suite target,
   * `suiteId/testName`, or a bare test name (substring); omitted runs all.
   */
  async runTests(filter?: string): Promise<TestRunReport> {
    return this.send<TestRunReport>({
      command: 'runTests',
      ...(filter ? { filter } : {}),
    });
  }

  async componentCoverage(): Promise<ComponentCoverageResponse> {
    return this.send<ComponentCoverageResponse>({ command: 'componentCoverage' });
  }

  async componentIncrementalScan(opts: {
    dirtyPaths?: string[];
    detectChanges?: boolean;
    force?: boolean;
  } = {}): Promise<ComponentScanResponse> {
    return this.send<ComponentScanResponse>({
      command: 'componentIncrementalScan',
      dirtyPaths: opts.dirtyPaths ?? [],
      detectChanges: opts.detectChanges ?? false,
      force: opts.force ?? false,
    });
  }

  async describeComponent(componentId: string): Promise<ComponentDetailSchema> {
    const response = await this.send<DescribeComponentResponse>({
      command: 'describeComponent',
      componentId,
    });
    if (!response.ok || !response.component) {
      throw new Error(response.error ?? `Failed to describe component ${componentId}`);
    }
    return response.component;
  }

  async componentScaffold(opts: {
    componentId: string;
    args?: ArgValues;
    preview?: boolean;
  }): Promise<ComponentInvokeResponse> {
    return this.send<ComponentInvokeResponse>({
      command: 'componentScaffold',
      componentId: opts.componentId,
      args: opts.args ?? {},
      preview: opts.preview ?? true,
    });
  }

  async componentInvokeOperation(opts: {
    instanceId: string;
    operation: string;
    args?: ArgValues;
    preview?: boolean;
  }): Promise<ComponentInvokeResponse> {
    return this.send<ComponentInvokeResponse>({
      command: 'componentInvokeOperation',
      instanceId: opts.instanceId,
      operation: opts.operation,
      args: opts.args ?? {},
      preview: opts.preview ?? true,
    });
  }

  async componentInvokeHook(opts: {
    componentId: string;
    hook: string;
    preview?: boolean;
    locator?: unknown;
    instance?: unknown;
    operation?: string;
    args?: ArgValues;
    previewToken?: string;
  }): Promise<ComponentInvokeResponse> {
    return this.send<ComponentInvokeResponse>({
      command: 'componentInvokeHook',
      componentId: opts.componentId,
      hook: opts.hook,
      preview: opts.preview ?? true,
      locator: opts.locator,
      instance: opts.instance,
      operation: opts.operation,
      args: opts.args ?? {},
      previewToken: opts.previewToken,
    });
  }

  async stackInvokeOperation(opts: {
    operation: string;
    args?: ArgValues;
    preview?: boolean;
    previewToken?: string;
  }): Promise<ComponentInvokeResponse> {
    return this.send<ComponentInvokeResponse>({
      command: 'stackInvokeOperation',
      operation: opts.operation,
      args: opts.args ?? {},
      preview: opts.preview ?? true,
      previewToken: opts.previewToken,
    });
  }

}
