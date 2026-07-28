import type { RecipeSchema } from '../shared';

export type RecipeTreeNode = {
  key: string;
  label: string;
  recipes: RecipeSchema[];
  children: RecipeTreeNode[];
};

export function recipeIdSegments(id: string): string[] {
  return id
    .split('.')
    .map((part) => part.trim())
    .filter(Boolean);
}

export function recipeLeafId(id: string): string {
  const segments = recipeIdSegments(id);
  return segments[segments.length - 1] ?? id;
}

export function recipeGroupPath(id: string): string[] {
  const segments = recipeIdSegments(id);
  return segments.slice(0, -1);
}

export function recipeDisplayTitle(recipe: RecipeSchema): string {
  return recipe.name === recipe.id ? recipeLeafId(recipe.id) : recipe.name;
}

export function buildRecipeTree(recipes: readonly RecipeSchema[]): RecipeTreeNode[] {
  const root: RecipeTreeNode = {
    key: '',
    label: '',
    recipes: [],
    children: [],
  };

  const ensureChild = (parent: RecipeTreeNode, segment: string, key: string) => {
    let child = parent.children.find((item) => item.label === segment);
    if (!child) {
      child = { key, label: segment, recipes: [], children: [] };
      parent.children.push(child);
    }
    return child;
  };

  for (const recipe of recipes) {
    const parts = recipeGroupPath(recipe.id);
    if (parts.length === 0) {
      const ungrouped = ensureChild(root, '(ungrouped)', '(ungrouped)');
      ungrouped.recipes.push(recipe);
      continue;
    }
    let node = root;
    let pathKey = '';
    for (const part of parts) {
      pathKey = pathKey ? `${pathKey}.${part}` : part;
      node = ensureChild(node, part, pathKey);
    }
    node.recipes.push(recipe);
  }

  const sortNode = (node: RecipeTreeNode) => {
    node.children.sort((a, b) => {
      if (a.label === '(ungrouped)') return 1;
      if (b.label === '(ungrouped)') return -1;
      return a.label.localeCompare(b.label);
    });
    node.recipes.sort((a, b) =>
      recipeDisplayTitle(a).localeCompare(recipeDisplayTitle(b))
    );
    for (const child of node.children) {
      sortNode(child);
    }
  };
  sortNode(root);
  return root.children;
}
