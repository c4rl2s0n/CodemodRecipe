import { describe, expect, it } from 'vitest';
import type { TestSuiteSchema } from '../../shared';
import { buildTestTree, collectLeaves, topLevelSelected, TestTreeNode } from './testTree';

function suite(id: string, testNames: string[]): TestSuiteSchema {
  return { id, testNames, path: `.codemod/components/${id}.test.luau` };
}

const SUITES: TestSuiteSchema[] = [
  suite('flutter.model.tests', ['add_store_field', 'parses_fields']),
  suite('flutter.stack.tests', ['store_relations_resolve']),
  suite('recipes', ['applies']),
];

function byId(roots: TestTreeNode[]): Map<string, TestTreeNode> {
  const map = new Map<string, TestTreeNode>();
  const walk = (node: TestTreeNode): void => {
    map.set(node.id, node);
    node.children.forEach(walk);
  };
  roots.forEach(walk);
  return map;
}

describe('buildTestTree', () => {
  it('groups suites by dot-separated id segments', () => {
    const roots = buildTestTree(SUITES);
    // flutter (group) + recipes (root-level suite)
    expect(roots.map((n) => n.id)).toEqual(['g:flutter', 's:recipes']);

    const flutter = roots[0];
    expect(flutter.kind).toBe('group');
    expect(flutter.filter).toBe('flutter');
    expect(flutter.children.map((n) => n.id)).toEqual(['g:flutter.model', 'g:flutter.stack']);

    const model = flutter.children[0];
    expect(model.children.map((n) => n.id)).toEqual(['s:flutter.model.tests']);

    const suiteNode = model.children[0];
    expect(suiteNode.kind).toBe('suite');
    expect(suiteNode.filter).toBe('flutter.model.tests');
    expect(suiteNode.children.map((n) => n.id)).toEqual([
      't:flutter.model.tests/add_store_field',
      't:flutter.model.tests/parses_fields',
    ]);
  });

  it('keeps host-sorted test order within a suite', () => {
    const roots = buildTestTree(SUITES);
    const map = byId(roots);
    const leaves = collectLeaves(map.get('s:flutter.model.tests')!);
    expect(leaves.map((n) => n.label)).toEqual(['add_store_field', 'parses_fields']);
    expect(leaves[0].filter).toBe('flutter.model.tests/add_store_field');
  });

  it('handles a single suite without dots at the root', () => {
    const roots = buildTestTree([suite('smoke', ['works'])]);
    expect(roots).toHaveLength(1);
    expect(roots[0].kind).toBe('suite');
    expect(roots[0].children[0].filter).toBe('smoke/works');
  });
});

describe('topLevelSelected', () => {
  it('drops items covered by a selected ancestor', () => {
    const roots = buildTestTree(SUITES);
    const map = byId(roots);
    const selected = topLevelSelected(
      ['g:flutter', 's:flutter.model.tests', 't:flutter.stack.tests/store_relations_resolve'],
      map
    );
    expect(selected.map((n) => n.id)).toEqual(['g:flutter']);
  });

  it('keeps disjoint selections', () => {
    const roots = buildTestTree(SUITES);
    const map = byId(roots);
    const selected = topLevelSelected(
      ['t:flutter.model.tests/parses_fields', 's:recipes'],
      map
    );
    expect(selected.map((n) => n.filter)).toEqual([
      'flutter.model.tests/parses_fields',
      'recipes',
    ]);
  });
});

describe('collectLeaves', () => {
  it('collects all tests under a group', () => {
    const roots = buildTestTree(SUITES);
    const map = byId(roots);
    const leaves = collectLeaves(map.get('g:flutter')!);
    expect(leaves.map((n) => n.id)).toEqual([
      't:flutter.model.tests/add_store_field',
      't:flutter.model.tests/parses_fields',
      't:flutter.stack.tests/store_relations_resolve',
    ]);
  });
});
