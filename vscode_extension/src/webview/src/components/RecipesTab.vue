<script setup lang="ts">
import { WEBVIEW_TO_EXTENSION, type RecipeSchema } from '../shared';
import { postToExtension } from '../vsCodeApi';

defineProps<{
  recipes: readonly RecipeSchema[];
  discoveryError?: string;
  refreshing: boolean;
}>();

function selectRecipe(id: string) {
  postToExtension({ type: WEBVIEW_TO_EXTENSION.selectRecipe, id });
}

function refresh() {
  postToExtension({ type: WEBVIEW_TO_EXTENSION.refreshRecipes });
}

function configureHost() {
  postToExtension({ type: WEBVIEW_TO_EXTENSION.configureHost });
}

function recipeSubtitle(recipe: RecipeSchema): string {
  return (
    recipe.description ||
    recipe.args.map((arg) => arg.name).join(', ')
  );
}
</script>

<template>
  <div>
    <h2>Recipes</h2>
    <div v-if="!recipes.length" class="empty-state">
      <p class="desc">
        {{ discoveryError ? 'Recipe discovery failed.' : 'No recipes found.' }}
      </p>
      <code v-if="discoveryError">{{ discoveryError }}</code>
      <div class="empty-actions">
        <button type="button" :disabled="refreshing" @click="refresh">
          {{ refreshing ? 'Refreshing…' : 'Refresh' }}
        </button>
        <button type="button" class="secondary" @click="configureHost">
          Set Host Entry Point
        </button>
      </div>
    </div>
    <div v-else class="recipe-list">
      <div v-if="refreshing" class="desc" style="margin-bottom: 8px">
        Refreshing recipes…
      </div>
      <button
        v-for="item in recipes"
        :key="item.id"
        type="button"
        class="recipe-button secondary"
        @click="selectRecipe(item.id)"
      >
        <span class="recipe-title">{{ item.name }}</span>
        <span class="recipe-desc">{{ recipeSubtitle(item) }}</span>
      </button>
    </div>
  </div>
</template>
