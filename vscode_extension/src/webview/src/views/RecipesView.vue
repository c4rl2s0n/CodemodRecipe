<script setup lang="ts">
import RecipesTab from '../components/RecipesTab.vue';
import type {
  ContextRecipeMatch,
  RecipeDiagnostic,
  RecipeSchema,
} from '../shared.js';

defineProps<{
  recipes: readonly RecipeSchema[];
  discoveryError?: string;
  diagnostics: readonly RecipeDiagnostic[];
  refreshing: boolean;
  contextMatches: readonly ContextRecipeMatch[];
  slotsByRecipe: Record<string, string[]>;
}>();
</script>

<template>
  <RecipesTab
    :recipes="recipes"
    :discovery-error="discoveryError"
    :diagnostics="diagnostics"
    :refreshing="refreshing"
    :context-matches="contextMatches"
    :slots-by-recipe="slotsByRecipe"
  />
</template>
