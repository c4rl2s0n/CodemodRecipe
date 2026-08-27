<script setup lang="ts">
import { watch } from 'vue';
import { argsKey, formatInputKind } from '../lib/args';
import type { ArgValues, RecipeArg, RecipeSchema } from '../shared';
import ArgField from './ArgField.vue';

defineProps<{
  recipe: RecipeSchema | undefined;
}>();

const argValues = defineModel<ArgValues>('argValues', {
  required: true,
});

const emit = defineEmits<{
  'args-changed': [];
  'submit-preview': [];
}>();

let lastEmittedKey = argsKey(argValues.value);

watch(
  argValues,
  (next) => {
    const key = argsKey(next);
    if (key === lastEmittedKey) {
      return;
    }
    lastEmittedKey = key;
    emit('args-changed');
  },
  { deep: true }
);

function paramMeta(arg: RecipeArg): string {
  const parts: string[] = [formatInputKind(arg.inputKind)];
  if (arg.contextKey) {
    parts.push('from ' + arg.contextKey);
  }
  if (arg.options?.length) {
    parts.push(arg.options.length + ' suggestions');
  }
  if (arg.fromRecipes?.length) {
    parts.push('nested: ' + arg.fromRecipes.join(', '));
  }
  return parts.join(' · ');
}
</script>

<template>
  <div v-if="recipe?.args?.length" class="param-list">
    <div v-for="arg in recipe.args" :key="arg.name" class="param">
      <span class="param-name">{{ arg.name }}{{ arg.required ? ' *' : '' }}</span>
      <span>{{ paramMeta(arg) }}</span>
    </div>
  </div>

  <div v-if="!recipe">
    <p class="desc">Choose a recipe from the Recipes tab.</p>
  </div>
  <template v-else>
    <div data-recipe-form class="recipe-form">
      <ArgField
        v-for="arg in recipe.args"
        :key="arg.name"
        v-model="argValues[arg.name]"
        :arg="arg"
        @submit-preview="emit('submit-preview')"
      />
    </div>
  </template>
</template>
