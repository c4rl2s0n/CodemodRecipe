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
  implements vscode.TreeDataProvider<AstTreeItem>, vscode.Disposable
{
  private readonly _onDidChange =
    new vscode.EventEmitter<AstTreeItem | undefined | void>();
  readonly onDidChangeTreeData = this._onDidChange.event;
  private root: AstNodeDto | undefined;
  private hasError = false;

  setRoot(root: AstNodeDto | undefined, hasError: boolean): void {
    this.root = root;
    this.hasError = hasError;
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

  getChildren(element?: AstTreeItem): AstTreeItem[] {
    if (!this.root) {
      return [];
    }
    if (!element) {
      return this.root.children.map(
        (c, i) => new AstTreeItem(c, [i])
      );
    }
    return element.node.children.map(
      (c, i) => new AstTreeItem(c, [...element.path, i])
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
