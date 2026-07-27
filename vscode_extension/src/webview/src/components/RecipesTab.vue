<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import type { RecipeDiagnostic, RecipeSchema } from '../shared';
import { useExtensionClient } from '../composables/useExtensionClient';
import RecipeGroupNode, { type RecipeTreeNode } from './RecipeGroupNode.vue';

const client = useExtensionClient();

const props = defineProps<{
  recipes: readonly RecipeSchema[];
  discoveryError?: string;
  diagnostics: readonly RecipeDiagnostic[];
  refreshing: boolean;
}>();

const searchQuery = ref('');
const collapsedGroups = ref<Record<string, boolean>>({});

type RecipeContextMenuState = {
  recipeId: string;
  x: number;
  y: number;
};

const recipeContextMenu = ref<RecipeContextMenuState | null>(null);

const errorDiagnostics = computed(() =>
  props.diagnostics.filter((item) => item.severity === 'error')
);
const warningDiagnostics = computed(() =>
  props.diagnostics.filter((item) => item.severity === 'warning')
);

const filteredRecipes = computed(() => {
  const q = searchQuery.value.trim().toLowerCase();
  if (!q) {
    return props.recipes;
  }
  return props.recipes.filter((recipe) => {
    const haystack = [
      recipe.id,
      recipe.name,
      recipe.description,
      recipe.group ?? '',
    ]
      .join(' ')
      .toLowerCase();
    return haystack.includes(q);
  });
});

const recipeTree = computed((): RecipeTreeNode[] => {
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

  for (const recipe of filteredRecipes.value) {
    const group = (recipe.group ?? '').trim();
    if (!group) {
      const ungrouped = ensureChild(root, '(ungrouped)', '(ungrouped)');
      ungrouped.recipes.push(recipe);
      continue;
    }
    const parts = group.split('.').filter(Boolean);
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
    node.recipes.sort((a, b) => a.name.localeCompare(b.name));
    for (const child of node.children) {
      sortNode(child);
    }
  };
  sortNode(root);
  return root.children;
});

function countRecipes(node: RecipeTreeNode): number {
  return (
    node.recipes.length +
    node.children.reduce((sum, child) => sum + countRecipes(child), 0)
  );
}

function isCollapsed(key: string): boolean {
  // Groups start collapsed; only expand when the user toggles them open.
  return collapsedGroups.value[key] !== false;
}

function toggleGroup(key: string) {
  collapsedGroups.value = {
    ...collapsedGroups.value,
    [key]: !isCollapsed(key),
  };
}

function selectRecipe(id: string) {
  client.selectRecipe(id);
}

function closeRecipeContextMenu(): void {
  recipeContextMenu.value = null;
}

function onRecipeContextMenu(recipe: RecipeSchema, event: MouseEvent): void {
  if (!recipe.sourceFile) {
    return;
  }
  event.preventDefault();
  recipeContextMenu.value = {
    recipeId: recipe.id,
    x: event.clientX,
    y: event.clientY,
  };
}

function showRecipeFromContextMenu(): void {
  const id = recipeContextMenu.value?.recipeId;
  if (id) {
    client.openRecipeFile(id);
  }
  closeRecipeContextMenu();
}

onMounted(() => {
  window.addEventListener('click', closeRecipeContextMenu);
  window.addEventListener('scroll', closeRecipeContextMenu, true);
});

onUnmounted(() => {
  window.removeEventListener('click', closeRecipeContextMenu);
  window.removeEventListener('scroll', closeRecipeContextMenu, true);
});

function refresh() {
  client.refreshRecipes();
}

function configureHost() {
  client.configureHost();
}

function scaffoldProject() {
  client.scaffoldProject();
}

function recipeSubtitle(recipe: RecipeSchema): string {
  return (
    recipe.description ||
    recipe.args.map((arg) => arg.name).join(', ')
  );
}

function formatSource(diagnostic: RecipeDiagnostic): string {
  const source = diagnostic.sources?.[0];
  if (!source) {
    return '';
  }
  const location =
    source.line != null
      ? `${source.file}:${source.line}`
      : source.file;
  return location;
}
</script>

<template>
  <div>
    <h2>Recipes</h2>

    <div v-if="errorDiagnostics.length" class="diagnostics diagnostics-errors">
      <h3>Recipe errors</h3>
      <div
        v-for="(item, index) in errorDiagnostics"
        :key="`error-${item.code}-${index}`"
        class="diagnostic-item diagnostic-error"
      >
        <div class="diagnostic-head">
          <span class="diagnostic-code">{{ item.code }}</span>
          <code v-if="formatSource(item)" class="diagnostic-source">{{ formatSource(item) }}</code>
        </div>
        <p class="diagnostic-message">{{ item.message }}</p>
      </div>
    </div>

    <div v-if="warningDiagnostics.length" class="diagnostics diagnostics-warnings">
      <h3>Recipe warnings</h3>
      <div
        v-for="(item, index) in warningDiagnostics"
        :key="`warning-${item.code}-${index}`"
        class="diagnostic-item diagnostic-warning"
      >
        <div class="diagnostic-head">
          <span class="diagnostic-code">{{ item.code }}</span>
          <code v-if="formatSource(item)" class="diagnostic-source">{{ formatSource(item) }}</code>
        </div>
        <p class="diagnostic-message">{{ item.message }}</p>
      </div>
    </div>

    <div v-if="!recipes.length" class="empty-state">
      <p class="desc">
        {{
          discoveryError
            ? 'Recipe discovery failed.'
            : errorDiagnostics.length
              ? 'Recipes could not be loaded due to errors above.'
              : 'No recipes found.'
        }}
      </p>
      <code v-if="discoveryError">{{ discoveryError }}</code>
      <div class="empty-actions">
        <button type="button" :disabled="refreshing" @click="refresh">
          {{ refreshing ? 'Refreshing…' : 'Refresh' }}
        </button>
        <button type="button" class="secondary" @click="configureHost">
          Set Codemod Root Directory
        </button>
        <button type="button" class="secondary" @click="scaffoldProject">
          Scaffold .codemod
        </button>
      </div>
    </div>
    <div v-else class="recipe-browser">
      <input
        v-model="searchQuery"
        type="text"
        class="recipe-search"
        placeholder="Search recipes…"
        aria-label="Search recipes"
      />
      <div v-if="!filteredRecipes.length" class="empty-state">
        <p class="desc">No recipes match “{{ searchQuery }}”.</p>
      </div>
      <div v-else class="recipe-tree">
        <RecipeGroupNode
          v-for="node in recipeTree"
          :key="node.key"
          :node="node"
          :depth="0"
          :is-collapsed="isCollapsed"
          :toggle-group="toggleGroup"
          :count-recipes="countRecipes"
          :select-recipe="selectRecipe"
          :on-recipe-context-menu="onRecipeContextMenu"
          :recipe-subtitle="recipeSubtitle"
        />
      </div>
    </div>

    <Teleport to="body">
      <div
        v-if="recipeContextMenu"
        class="recipe-context-menu"
        :style="{
          left: `${recipeContextMenu.x}px`,
          top: `${recipeContextMenu.y}px`,
        }"
        role="menu"
        @click.stop
      >
        <button
          type="button"
          class="recipe-context-menu-item"
          role="menuitem"
          @click="showRecipeFromContextMenu"
        >
          Show Recipe
        </button>
      </div>
    </Teleport>
  </div>
</template>
