import * as vscode from 'vscode';
import type { AstNodeDto } from '../../shared';

export class AstTreeItem extends vscode.TreeItem {
  constructor(
    public readonly node: AstNodeDto,
    public readonly path: number[]
  ) {
    const field = node.field ? `${node.field}: ` : '';
    const err = node.isError ? ' ⚠' : '';
    const label = `${field}${node.kind}${err}`;
    super(
      label,
      node.children.length > 0
        ? vscode.TreeItemCollapsibleState.Collapsed
        : vscode.TreeItemCollapsibleState.None
    );
    this.id = path.join('.');
    this.description = `[${node.start.line},${node.start.column}]–[${node.end.line},${node.end.column}]`;
    this.tooltip = node.text
      ? `${label}\n${node.text}`
      : `${label} bytes ${node.start.byte}-${node.end.byte}`;
    this.contextValue = 'astNode';
    this.command = {
      command: 'codemodRecipe.queryTools.selectAstNode',
      title: 'Select AST node',
      arguments: [this],
    };
    if (node.isError) {
      this.iconPath = new vscode.ThemeIcon('warning');
    }
  }
}

export class AstTreeProvider
  implements
    vscode.TreeDataProvider<AstTreeItem>,
    vscode.Disposable
{
  private readonly _onDidChange =
    new vscode.EventEmitter<AstTreeItem | undefined | void>();
  readonly onDidChangeTreeData = this._onDidChange.event;
  private root: AstNodeDto | undefined;
  private hasError = false;
  private readonly itemCache = new Map<string, AstTreeItem>();

  setRoot(root: AstNodeDto | undefined, hasError: boolean): void {
    this.root = root;
    this.hasError = hasError;
    this.itemCache.clear();
    this._onDidChange.fire();
  }

  getHasError(): boolean {
    return this.hasError;
  }

  getRoot(): AstNodeDto | undefined {
    return this.root;
  }

  findPathForByte(byte: number): number[] | undefined {
    if (!this.root) {
      return undefined;
    }
    return findPath(this.root, byte, []);
  }

  getItemForPath(pathIdx: number[]): AstTreeItem | undefined {
    if (!this.root) {
      return undefined;
    }
    const key = pathIdx.join('.');
    const cached = this.itemCache.get(key);
    if (cached) {
      return cached;
    }
    let node: AstNodeDto | undefined = this.root;
    for (const i of pathIdx) {
      node = node?.children[i];
    }
    if (!node) {
      return undefined;
    }
    const item = new AstTreeItem(node, pathIdx);
    this.itemCache.set(key, item);
    return item;
  }

  getParent(element: AstTreeItem): AstTreeItem | undefined {
    if (element.path.length <= 1) {
      return undefined;
    }
    return this.getItemForPath(element.path.slice(0, -1));
  }

  getChildren(element?: AstTreeItem): AstTreeItem[] {
    if (!this.root) {
      return [];
    }
    if (!element) {
      return this.root.children.map((_, i) => this.getItemForPath([i])!);
    }
    return element.node.children.map((_, i) =>
      this.getItemForPath([...element.path, i])!
    );
  }

  getTreeItem(element: AstTreeItem): vscode.TreeItem {
    return element;
  }

  dispose(): void {
    this._onDidChange.dispose();
  }
}

function findPath(
  node: AstNodeDto,
  byte: number,
  path: number[]
): number[] | undefined {
  if (byte < node.start.byte || byte > node.end.byte) {
    return undefined;
  }
  for (let i = 0; i < node.children.length; i++) {
    const child = node.children[i];
    if (byte >= child.start.byte && byte <= child.end.byte) {
      const deeper = findPath(child, byte, [...path, i]);
      return deeper ?? [...path, i];
    }
  }
  return path.length ? path : undefined;
}
