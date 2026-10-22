import {
  diffModifiedKey,
  diffOriginalKey,
} from '../../diff/diffContentProvider';
import type { ArgValues, ComponentDetailSchema } from '../../../shared';
import type { RecipeRunnerHandlerHost } from './types';

export async function describeComponentOrError(
  host: RecipeRunnerHandlerHost,
  componentId: string
): Promise<{ detail?: ComponentDetailSchema; error?: string }> {
  const repository = host.recipeRepository;
  if (!repository) {
    return { error: 'Recipe repository is not available' };
  }
  try {
    const detail = await repository.describeComponentCached(componentId);
    if (!detail) {
      return {
        error: `Component "${componentId}" could not be described`,
      };
    }
    return { detail };
  } catch (err) {
    return {
      error: err instanceof Error ? err.message : String(err),
    };
  }
}

export async function handleSelectComponent(
  host: RecipeRunnerHandlerHost,
  componentId: string
): Promise<void> {
  const { detail, error } = await describeComponentOrError(host, componentId);
  host.state.selectComponent(componentId, detail);
  if (error) {
    host.state.componentOpError = error;
  }
  host.postState();
}

export async function handleSelectComponentInstance(
  host: RecipeRunnerHandlerHost,
  instanceId: string
): Promise<void> {
  const instance = host.state.componentInstances.find((i) => i.id === instanceId);
  if (!instance) {
    return;
  }
  if (instance.componentId !== host.state.selectedComponentId) {
    const { detail, error } = await describeComponentOrError(
      host,
      instance.componentId
    );
    host.state.selectComponent(instance.componentId, detail);
    host.state.selectComponentInstance(instanceId);
    if (error) {
      host.state.componentOpError = error;
    }
  } else {
    host.state.selectComponentInstance(instanceId);
  }
  host.postState();
}

export async function handleComponentPreview(
  host: RecipeRunnerHandlerHost,
  args: ArgValues,
  _requestId: number,
  preview: boolean
): Promise<void> {
  const componentId = host.state.selectedComponentId;
  const action = host.state.componentAction;
  if (!componentId || !action) {
    host.state.setComponentPreviewResult(
      [],
      [],
      'Select a component action first'
    );
    host.postState();
    return;
  }
  host.state.componentArgs = args;
  try {
    let response;
    if (action === 'scaffold') {
      response = await host.bridge.componentScaffold({
        componentId,
        args,
        preview,
      });
    } else {
      const instanceId = host.state.selectedInstanceId;
      if (!instanceId) {
        host.state.setComponentPreviewResult(
          [],
          [],
          'Select an instance to run this operation'
        );
        host.postState();
        return;
      }
      response = await host.bridge.componentInvokeOperation({
        instanceId,
        operation: action,
        args,
        preview,
      });
    }
    if (!response.ok) {
      host.state.setComponentPreviewResult(
        [],
        response.transcript ?? [],
        response.error ?? 'Component operation failed'
      );
      host.postState();
      return;
    }
    const files = response.files ?? [];
    for (const file of files) {
      const originalKey = diffOriginalKey(file.path);
      const modifiedKey = diffModifiedKey(file.path);
      host.diffProvider.store(originalKey, file.original ?? '');
      host.diffProvider.store(modifiedKey, file.modified ?? '');
    }
    host.state.setComponentPreviewResult(files, response.transcript ?? []);
    if (!preview && host.scanComponents) {
      await host.scanComponents();
    } else {
      host.postState();
    }
  } catch (err) {
    host.state.setComponentPreviewResult(
      [],
      [],
      err instanceof Error ? err.message : String(err)
    );
    host.postState();
  }
}

export async function handleStackOperation(
  host: RecipeRunnerHandlerHost,
  args: ArgValues,
  _requestId: number,
  preview: boolean
): Promise<void> {
  const operation = host.state.stackAction;
  if (!operation) {
    host.state.setComponentPreviewResult([], [], 'Select a stack operation first');
    host.postState();
    return;
  }
  host.state.componentArgs = args;
  try {
    let previewToken: string | undefined;
    if (!preview) {
      const dry = await host.bridge.stackInvokeOperation({
        operation,
        args,
        preview: true,
      });
      if (!dry.ok || !dry.previewToken) {
        host.state.setComponentPreviewResult(
          [],
          dry.transcript ?? [],
          dry.error ?? 'Stack dry-run failed (previewToken required before apply)'
        );
        host.postState();
        return;
      }
      previewToken = dry.previewToken;
    }
    const response = await host.bridge.stackInvokeOperation({
      operation,
      args,
      preview,
      previewToken,
    });
    if (!response.ok) {
      host.state.setComponentPreviewResult(
        [],
        response.transcript ?? [],
        response.error ?? 'Stack operation failed'
      );
      host.postState();
      return;
    }
    const files = response.files ?? [];
    for (const file of files) {
      host.diffProvider.store(diffOriginalKey(file.path), file.original ?? '');
      host.diffProvider.store(diffModifiedKey(file.path), file.modified ?? '');
    }
    host.state.setComponentPreviewResult(files, response.transcript ?? []);
    if (!preview && host.scanComponents) {
      await host.scanComponents();
    } else {
      host.postState();
    }
  } catch (err) {
    host.state.setComponentPreviewResult(
      [],
      [],
      err instanceof Error ? err.message : String(err)
    );
    host.postState();
  }
}

export async function handleComponentDryRun(
  host: RecipeRunnerHandlerHost,
  hook: string,
  _requestId: number
): Promise<void> {
  const componentId = host.state.selectedComponentId;
  if (!componentId) {
    host.state.setComponentDryRunResult(hook, undefined, [], [], 'Select a component first');
    host.postState();
    return;
  }

  const selected = host.state.componentInstances.find(
    (i) => i.id === host.state.selectedInstanceId
  );

  let locator: unknown | undefined;
  let instance: unknown | undefined;
  let operation: string | undefined;
  let args: ArgValues = {};

  if (hook === 'parse') {
    if (!selected) {
      host.state.setComponentDryRunResult(
        hook,
        undefined,
        [],
        [],
        'Select an instance to dry-run parse (uses its locator)'
      );
      host.postState();
      return;
    }
    locator = selected.locator;
  } else if (hook === 'verify' || hook === 'toSummary') {
    if (!selected) {
      host.state.setComponentDryRunResult(
        hook,
        undefined,
        [],
        [],
        `Select an instance to dry-run ${hook}`
      );
      host.postState();
      return;
    }
    instance = {
      id: selected.id,
      componentId: selected.componentId,
      locator: selected.locator,
      ast: selected.ast,
    };
  } else if (hook === 'scaffold') {
    args = host.state.componentArgs;
  } else if (hook === 'operation') {
    const opName = host.state.componentAction;
    if (!opName || opName === 'scaffold') {
      host.state.setComponentDryRunResult(
        hook,
        undefined,
        [],
        [],
        'Select an operation action first'
      );
      host.postState();
      return;
    }
    if (!selected) {
      host.state.setComponentDryRunResult(
        hook,
        undefined,
        [],
        [],
        'Select an instance to dry-run this operation'
      );
      host.postState();
      return;
    }
    operation = opName;
    instance = {
      id: selected.id,
      componentId: selected.componentId,
      locator: selected.locator,
      ast: selected.ast,
    };
    args = host.state.componentArgs;
  }

  try {
    const response = await host.bridge.componentInvokeHook({
      componentId,
      hook,
      preview: true,
      locator,
      instance,
      operation,
      args,
    });
    if (!response.ok) {
      host.state.setComponentDryRunResult(
        hook,
        response.result,
        [],
        response.transcript ?? [],
        response.error ?? 'Component dry-run failed'
      );
      host.postState();
      return;
    }
    const files = response.files ?? [];
    for (const file of files) {
      host.diffProvider.store(diffOriginalKey(file.path), file.original ?? '');
      host.diffProvider.store(diffModifiedKey(file.path), file.modified ?? '');
    }
    host.state.setComponentDryRunResult(
      hook,
      response.result,
      files,
      response.transcript ?? []
    );
    host.postState();
  } catch (err) {
    host.state.setComponentDryRunResult(
      hook,
      undefined,
      [],
      [],
      err instanceof Error ? err.message : String(err)
    );
    host.postState();
  }
}

