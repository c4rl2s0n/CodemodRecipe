import * as vscode from 'vscode';
import { COMMANDS } from '../../constants';
import {
  WEBVIEW_TO_EXTENSION,
  assertNever,
  type WebviewToExtensionMessage,
} from '../../../shared';
import type { RecipeRunnerHandlerHost } from './types';
import {
  handleCreateShortcut,
  handleInvokeRecipe,
  handleSelectRecipe,
} from './recipes';
import {
  handleApply,
  handleOpenRecipeFile,
  handlePickPath,
  handlePreview,
} from './previewApply';
import { handleOpenDiffByPath } from './diffPreview';
import {
  handleComponentDryRun,
  handleComponentPreview,
  handleSelectComponent,
  handleSelectComponentInstance,
  handleStackOperation,
} from './components';

export type { RecipeRunnerHandlerHost } from './types';

export async function handleWebviewMessage(
  host: RecipeRunnerHandlerHost,
  message: WebviewToExtensionMessage
): Promise<void> {
  switch (message.type) {
    case WEBVIEW_TO_EXTENSION.ready:
      host.resetSession();
      host.postState();
      break;
    case WEBVIEW_TO_EXTENSION.bootstrapRetry:
      await vscode.commands.executeCommand(COMMANDS.bootstrap);
      break;
    case WEBVIEW_TO_EXTENSION.selectRecipe:
      await handleSelectRecipe(host, message.id);
      break;
    case WEBVIEW_TO_EXTENSION.refreshRecipes:
      await vscode.commands.executeCommand(COMMANDS.refresh);
      break;
    case WEBVIEW_TO_EXTENSION.configureHost:
      await vscode.commands.executeCommand(COMMANDS.configureCodemodRoot);
      break;
    case WEBVIEW_TO_EXTENSION.scaffoldProject:
      if (host.scaffoldHandler) {
        await host.scaffoldHandler();
      } else {
        await vscode.commands.executeCommand(COMMANDS.scaffoldProject);
      }
      break;
    case WEBVIEW_TO_EXTENSION.openRecipeFile:
      await handleOpenRecipeFile(host, message.id);
      break;
    case WEBVIEW_TO_EXTENSION.pickFile:
      await handlePickPath(host, message.arg, false, message.requestId);
      break;
    case WEBVIEW_TO_EXTENSION.pickDirectory:
      await handlePickPath(host, message.arg, true, message.requestId);
      break;
    case WEBVIEW_TO_EXTENSION.preview:
      // Do not await — must observe superseding preview messages while apply runs.
      void handlePreview(host, message.args, message.requestId);
      break;
    case WEBVIEW_TO_EXTENSION.openDiff:
      await handleOpenDiffByPath(host, message.path);
      break;
    case WEBVIEW_TO_EXTENSION.apply:
      await handleApply(host, message.selection);
      break;
    case WEBVIEW_TO_EXTENSION.invokeRecipe:
      await handleInvokeRecipe(
        host,
        message.recipeId,
        message.mode,
        message.args
      );
      break;
    case WEBVIEW_TO_EXTENSION.createShortcut:
      await handleCreateShortcut(host, message.recipeId, message.args);
      break;
    case WEBVIEW_TO_EXTENSION.selectComponent:
      await handleSelectComponent(host, message.componentId);
      break;
    case WEBVIEW_TO_EXTENSION.selectComponentInstance:
      await handleSelectComponentInstance(host, message.instanceId);
      break;
    case WEBVIEW_TO_EXTENSION.selectComponentAction:
      host.state.selectComponentAction(message.action);
      host.postState();
      break;
    case WEBVIEW_TO_EXTENSION.componentScan:
      if (host.scanComponents) {
        await host.scanComponents();
      }
      break;
    case WEBVIEW_TO_EXTENSION.componentCoverage:
      if (host.refreshComponentCoverage) {
        await host.refreshComponentCoverage();
      }
      break;
    case WEBVIEW_TO_EXTENSION.openCoverageGaps:
      if (host.openCoverageGaps) {
        await host.openCoverageGaps(message.path, message.gaps);
      }
      break;
    case WEBVIEW_TO_EXTENSION.clearCoverageGaps:
      host.clearCoverageGaps?.();
      break;
    case WEBVIEW_TO_EXTENSION.componentPreview:
      void handleComponentPreview(host, message.args, message.requestId, true);
      break;
    case WEBVIEW_TO_EXTENSION.componentApply:
      await handleComponentPreview(host, message.args, message.requestId, false);
      break;
    case WEBVIEW_TO_EXTENSION.componentDryRun:
      await handleComponentDryRun(host, message.hook, message.requestId);
      break;
    case WEBVIEW_TO_EXTENSION.selectStackOperation:
      host.state.selectStackOperation(message.operation);
      host.postState();
      break;
    case WEBVIEW_TO_EXTENSION.stackPreview:
      void handleStackOperation(host, message.args, message.requestId, true);
      break;
    case WEBVIEW_TO_EXTENSION.stackApply:
      await handleStackOperation(host, message.args, message.requestId, false);
      break;
    default:
      assertNever(message);
  }
}
