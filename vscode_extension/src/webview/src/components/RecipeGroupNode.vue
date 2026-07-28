<script setup lang="ts">
import type { RecipeSchema } from '../shared';
import type { RecipeTreeNode } from '../lib/recipeTree';

defineProps<{
  node: RecipeTreeNode;
  depth: number;
  isCollapsed: (key: string) => boolean;
  toggleGroup: (key: string) => void;
  countRecipes: (node: RecipeTreeNode) => number;
  selectRecipe: (id: string) => void;
  onRecipeContextMenu: (recipe: RecipeSchema, event: MouseEvent) => void;
  recipeSubtitle: (recipe: RecipeSchema) => string;
  recipeTitle: (recipe: RecipeSchema) => string;
}>();
</script>

<template>
  <div class="recipe-group" :class="{ nested: depth > 0 }" :style="{ '--depth': depth }">
    <button
      type="button"
      class="group-toggle secondary"
      @click="toggleGroup(node.key)"
    >
      <span class="group-chevron">{{ isCollapsed(node.key) ? '▸' : '▾' }}</span>
      <span class="group-label">{{ node.label }}</span>
      <span class="group-count">{{ countRecipes(node) }}</span>
    </button>
    <div v-if="!isCollapsed(node.key)" class="group-body">
      <div class="recipe-list">
        <div
          v-for="item in node.recipes"
          :key="item.id"
          class="recipe-row"
          @contextmenu="onRecipeContextMenu(item, $event)"
        >
          <button
            type="button"
            class="recipe-button secondary"
            @click="selectRecipe(item.id)"
          >
            <span class="recipe-title">{{ recipeTitle(item) }}</span>
            <span v-if="item.id.includes('.')" class="recipe-group-path">{{ item.id }}</span>
            <span class="recipe-desc">{{ recipeSubtitle(item) }}</span>
          </button>
        </div>
      </div>
      <RecipeGroupNode
        v-for="child in node.children"
        :key="child.key"
        :node="child"
        :depth="depth + 1"
        :is-collapsed="isCollapsed"
        :toggle-group="toggleGroup"
        :count-recipes="countRecipes"
        :select-recipe="selectRecipe"
        :on-recipe-context-menu="onRecipeContextMenu"
        :recipe-subtitle="recipeSubtitle"
        :recipe-title="recipeTitle"
      />
    </div>
  </div>
</template>

<script lang="ts">
export default {
  name: 'RecipeGroupNode',
};
</script>
