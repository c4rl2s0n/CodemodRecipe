<script setup lang="ts">
import { computed } from 'vue';
import { WEBVIEW_TO_EXTENSION, type RecipeDiagnostic, type RecipeSchema } from '../shared';
import { postToExtension } from '../vsCodeApi';

const props = defineProps<{
  recipes: readonly RecipeSchema[];
  discoveryError?: string;
  diagnostics: readonly RecipeDiagnostic[];
  refreshing: boolean;
}>();

const errorDiagnostics = computed(() =>
  props.diagnostics.filter((item) => item.severity === 'error')
);
const warningDiagnostics = computed(() =>
  props.diagnostics.filter((item) => item.severity === 'warning')
);

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
          Set Host Entry Point
        </button>
      </div>
    </div>
    <div v-else class="recipe-list">
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
