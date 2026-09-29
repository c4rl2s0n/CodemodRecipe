import * as fs from 'fs';
import * as path from 'path';
import * as vscode from 'vscode';
import type { TestRunReport, TestSuiteSchema } from '../../shared';
import { HostBridge } from '../host/hostBridge';
import { findTestFunctionLine } from './testSourceLocation';
import { buildTestTree, collectLeaves, topLevelSelected, TestTreeNode } from './testTree';

/**
 * Native Testing-view integration for `*.test.luau` suites. The tree is
 * rebuilt from the host catalog on every reload (the existing `.codemod`
 * watcher drives that); runs go through the host `runTests` command and the
 * JSON report is applied back onto the tree items.
 */
export class CodemodTestController {
  private readonly controller: vscode.TestController;
  private readonly profile: vscode.TestRunProfile;
  private nodeById = new Map<string, TestTreeNode>();
  private itemById = new Map<string, vscode.TestItem>();

  constructor(private readonly bridge: HostBridge) {
    this.controller = vscode.tests.createTestController('codemodRecipeTests', 'Codemod Recipe');
    this.profile = this.controller.createRunProfile(
      'Run',
      vscode.TestRunProfileKind.Run,
      (request, token) => this.executeRun(request, token),
      /* isDefault */ true
    );
  }

  dispose(): void {
    this.profile.dispose();
    this.controller.dispose();
  }

  /** Run every suite (palette command entry point). */
  async runAll(): Promise<void> {
    const cancellation = new vscode.CancellationTokenSource();
    try {
      await this.executeRun(new vscode.TestRunRequest(), cancellation.token);
    } finally {
      cancellation.dispose();
    }
  }

  /** Rebuild the TestItem tree from the host catalog. */
  async syncFromCatalog(
    suites: readonly TestSuiteSchema[],
    workspaceRoot: string
  ): Promise<void> {
    const roots = buildTestTree(suites);
    const uriBySuiteId = new Map<string, vscode.Uri>();
    for (const suite of suites) {
      if (suite.path) {
        // Catalog paths are workspace-relative in the standard layout
        // (`.codemod` inside the workspace root); tolerate absolute paths
        // from hosts configured with an external codemod root.
        const fsPath = path.isAbsolute(suite.path)
          ? suite.path
          : path.join(workspaceRoot, suite.path);
        uriBySuiteId.set(suite.id, vscode.Uri.file(fsPath));
      }
    }
    this.nodeById.clear();
    this.itemById.clear();
    this.controller.items.replace(
      roots.map((node) => this.toItem(node, uriBySuiteId))
    );
    // Best-effort per-test line locations (gutter icons, go-to-test). Failure
    // locations come from the host report instead — this is only for display.
    await Promise.all(
      suites.map(async (suite) => {
        const uri = uriBySuiteId.get(suite.id);
        if (!uri) {
          return;
        }
        let source: string;
        try {
          source = await fs.promises.readFile(uri.fsPath, 'utf8');
        } catch {
          return;
        }
        for (const name of suite.testNames) {
          const item = this.itemById.get(`t:${suite.id}/${name}`);
          if (!item) {
            continue;
          }
          const line = findTestFunctionLine(source, name);
          item.range = new vscode.Range(line, 0, line, 0);
        }
      })
    );
  }

  private toItem(node: TestTreeNode, uriBySuiteId: Map<string, vscode.Uri>): vscode.TestItem {
    const uri = node.suiteId ? uriBySuiteId.get(node.suiteId) : undefined;
    const item = this.controller.createTestItem(node.id, node.label, uri);
    this.nodeById.set(node.id, node);
    this.itemById.set(node.id, item);
    item.children.replace(node.children.map((child) => this.toItem(child, uriBySuiteId)));
    return item;
  }

  private async executeRun(
    request: vscode.TestRunRequest,
    token: vscode.CancellationToken
  ): Promise<void> {
    const run = this.controller.createTestRun(request);
    try {
      const selected = request.include
        ? topLevelSelected(
            [...request.include].map((item) => item.id),
            this.nodeById
          )
        : undefined;
      // One host call per top-level selection; run-all uses no filter.
      const filters = selected
        ? [...new Set(selected.map((node) => node.filter))]
        : [undefined];
      const leaves = selected
        ? selected.flatMap(collectLeaves)
        : [...this.nodeById.values()].filter((node) => node.kind === 'test');

      for (const leaf of leaves) {
        const item = this.itemById.get(leaf.id);
        if (item) {
          run.enqueued(item);
        }
      }

      const applied = new Set<string>();
      for (const filter of filters) {
        if (token.isCancellationRequested) {
          break;
        }
        let report: TestRunReport;
        try {
          report = await this.bridge.runTests(filter);
        } catch (err) {
          const message = err instanceof Error ? err.message : String(err);
          run.appendOutput(`runTests failed: ${message}\r\n`);
          for (const leaf of leaves) {
            this.errorItem(run, leaf, message);
            applied.add(leaf.id);
          }
          break;
        }
        this.applyReport(run, report, leaves, applied);
      }

      // Selected but never reported (cancelled, or suite vanished between
      // discovery and run): close out as skipped so no state dangles.
      for (const leaf of leaves) {
        if (!applied.has(leaf.id)) {
          const item = this.itemById.get(leaf.id);
          if (item) {
            run.skipped(item);
          }
        }
      }
    } finally {
      run.end();
    }
  }

  private applyReport(
    run: vscode.TestRun,
    report: TestRunReport,
    leaves: TestTreeNode[],
    applied: Set<string>
  ): void {
    // Usage-level failure (no suites, filter matched nothing): no per-test
    // results exist, so surface the message on every selected leaf.
    if (report.error) {
      run.appendOutput(`${report.error}\r\n`);
      for (const leaf of leaves) {
        this.errorItem(run, leaf, report.error);
        applied.add(leaf.id);
      }
      return;
    }
    for (const suite of report.suites) {
      for (const test of suite.tests) {
        const leafId = `t:${suite.id}/${test.name}`;
        const item = this.itemById.get(leafId);
        if (!item) {
          continue;
        }
        applied.add(leafId);
        if (test.ok) {
          run.passed(item);
          continue;
        }
        const message = new vscode.TestMessage(test.error ?? 'test failed');
        if (item.uri && test.line && test.line > 0) {
          message.location = new vscode.Location(
            item.uri,
            new vscode.Position(test.line - 1, 0)
          );
        }
        run.failed(item, message);
        if (test.error) {
          run.appendOutput(
            `${test.error.replace(/\n/g, '\r\n')}\r\n`,
            message.location,
            item
          );
        }
      }
    }
    run.appendOutput(
      `codemod tests: ${report.totals.passed} passed, ${report.totals.failed} failed\r\n`
    );
  }

  private errorItem(run: vscode.TestRun, leaf: TestTreeNode, message: string): void {
    const item = this.itemById.get(leaf.id);
    if (item) {
      run.errored(item, new vscode.TestMessage(message));
    }
  }
}
