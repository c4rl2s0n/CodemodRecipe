import type { TestSuiteSchema } from '../../shared';

export type TestNodeKind = 'group' | 'suite' | 'test';

/**
 * Pure tree model for the Testing view. Suite ids split on `.` into nested
 * group nodes; the suite is the last segment; test functions are leaves.
 *
 * Item ids are prefixed by kind (`g:` / `s:` / `t:`) so a suite id can never
 * collide with a group path. `filter` is the string passed to the host
 * `runTests` command when the node is run.
 */
export interface TestTreeNode {
  id: string;
  label: string;
  kind: TestNodeKind;
  filter: string;
  parentId?: string;
  suiteId?: string;
  testName?: string;
  children: TestTreeNode[];
}

export function buildTestTree(suites: readonly TestSuiteSchema[]): TestTreeNode[] {
  const roots: TestTreeNode[] = [];
  const groupByPath = new Map<string, TestTreeNode>();

  const insertSorted = (siblings: TestTreeNode[], node: TestTreeNode): void => {
    siblings.push(node);
    siblings.sort((a, b) => a.label.localeCompare(b.label));
  };

  const ensureGroup = (segments: string[]): TestTreeNode => {
    const path = segments.join('.');
    const existing = groupByPath.get(path);
    if (existing) {
      return existing;
    }
    const parentSegments = segments.slice(0, -1);
    const node: TestTreeNode = {
      id: `g:${path}`,
      label: segments[segments.length - 1],
      kind: 'group',
      // Substring filter: matches every suite id containing this path.
      filter: path,
      parentId: parentSegments.length === 0 ? undefined : `g:${parentSegments.join('.')}`,
      children: [],
    };
    groupByPath.set(path, node);
    const siblings = parentSegments.length === 0 ? roots : ensureGroup(parentSegments).children;
    insertSorted(siblings, node);
    return node;
  };

  for (const suite of [...suites].sort((a, b) => a.id.localeCompare(b.id))) {
    const segments = suite.id.split('.');
    const label = segments[segments.length - 1];
    const groupSegments = segments.slice(0, -1);
    const parentChildren = groupSegments.length > 0 ? ensureGroup(groupSegments).children : roots;
    const suiteNode: TestTreeNode = {
      id: `s:${suite.id}`,
      label,
      kind: 'suite',
      filter: suite.id,
      parentId: groupSegments.length > 0 ? `g:${groupSegments.join('.')}` : undefined,
      suiteId: suite.id,
      children: suite.testNames.map((testName) => ({
        id: `t:${suite.id}/${testName}`,
        label: testName,
        kind: 'test' as const,
        filter: `${suite.id}/${testName}`,
        parentId: `s:${suite.id}`,
        suiteId: suite.id,
        testName,
        children: [],
      })),
    };
    insertSorted(parentChildren, suiteNode);
  }
  return roots;
}

/** Depth-first leaf (`test`) nodes under `node`; a test node is its own leaf. */
export function collectLeaves(node: TestTreeNode): TestTreeNode[] {
  if (node.kind === 'test') {
    return [node];
  }
  return node.children.flatMap(collectLeaves);
}

/**
 * Drop selected nodes whose ancestor is also selected (running the ancestor
 * already covers them), so each host call runs a disjoint filter set.
 */
export function topLevelSelected(
  selectedIds: readonly string[],
  byId: Map<string, TestTreeNode>
): TestTreeNode[] {
  const selected = new Set(selectedIds);
  const out: TestTreeNode[] = [];
  for (const id of selected) {
    const node = byId.get(id);
    if (!node) {
      continue;
    }
    let ancestor = node.parentId;
    let covered = false;
    while (ancestor !== undefined) {
      if (selected.has(ancestor)) {
        covered = true;
        break;
      }
      ancestor = byId.get(ancestor)?.parentId;
    }
    if (!covered) {
      out.push(node);
    }
  }
  return out;
}
