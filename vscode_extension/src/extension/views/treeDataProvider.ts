import * as vscode from 'vscode';

class FileTreeItem extends vscode.TreeItem {
  constructor(
    public readonly label: string,
    public readonly uri: vscode.Uri,
    public readonly collapsibleState: vscode.TreeItemCollapsibleState,
  ) {
    super(label, collapsibleState);
    this.tooltip = this.uri.fsPath;
    this.description = this.uri.fsPath;
  }
  iconPath = new vscode.ThemeIcon('file');
}

export class FileTreeDataProvider implements vscode.TreeDataProvider<FileTreeItem> {
  private _onDidChangeTreeData: vscode.EventEmitter<FileTreeItem | undefined | null | void> = new vscode.EventEmitter();
  readonly onDidChangeTreeData: vscode.Event<FileTreeItem | undefined | null | void> = this._onDidChangeTreeData.event;

  constructor(private workspaceRoot: string) {}

  refresh(): void {
    this._onDidChangeTreeData.fire();
  }

  getTreeItem(element: FileTreeItem): vscode.TreeItem {
    return element;
  }

  getChildren(element?: FileTreeItem): Thenable<FileTreeItem[]> {
    if (!element) {
      // Root: list workspace folders
      const rootUri = vscode.Uri.file(this.workspaceRoot);
      return this.getChildrenForUri(rootUri);
    } else {
      // Subfolder: list its contents
      return this.getChildrenForUri(element.uri);
    }
  }

  private async getChildrenForUri(uri: vscode.Uri): Promise<FileTreeItem[]> {
    const children: FileTreeItem[] = [];
    try {
      const entries = await vscode.workspace.fs.readDirectory(uri);
      for (const [name, type] of entries) {
        const childUri = vscode.Uri.joinPath(uri, name);
        const treeItem = new FileTreeItem(
          name,
          childUri,
          type === vscode.FileType.Directory ? vscode.TreeItemCollapsibleState.Collapsed : vscode.TreeItemCollapsibleState.None,
        );
        children.push(treeItem);
      }
    } catch (err) {
      console.error(`Error reading directory ${uri.fsPath}:`, err);
    }
    return children;
  }
}