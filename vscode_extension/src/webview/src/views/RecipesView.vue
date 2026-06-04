<script setup lang="ts">
import RecipesTab from '../components/RecipesTab.vue';
import type { RecipeSchema } from '../shared.js';

defineProps<{
  recipes: readonly RecipeSchema[];
  discoveryError?: string;
  refreshing: boolean;
}>();
</script>

<template>
  <RecipesTab
    :recipes="recipes"
    :discovery-error="discoveryError"
    :refreshing="refreshing"
  />
</template>

