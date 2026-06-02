<script setup lang="ts">
import { watch } from 'vue';
import type { RecipeArg, RecipeSchema } from '../types';
import ArgField from './ArgField.vue';

defineProps<{
  recipe: RecipeSchema | undefined;
}>();

const argValues = defineModel<Record<string, string>>('argValues', {
  required: true,
});

const emit = defineEmits<{
  'args-changed': [];
  'submit-preview': [];
}>();

watch(
  argValues,
  () => {
    emit('args-changed');
  },
  { deep: true }
);

function paramMeta(arg: RecipeArg): string {
  const parts = [arg.inputKind || 'text'];
  if (arg.contextKey) {
    parts.push('from ' + arg.contextKey);
  }
  if (arg.options?.length) {
    parts.push(arg.options.length + ' suggestions');
  }
  return parts.join(' · ');
}
</script>

<template>
  <div v-if="recipe" class="param-list">
    <div v-for="arg in recipe.args" :key="arg.name" class="param">
      <span class="param-name">{{ arg.name }}{{ arg.required ? ' *' : '' }}</span>
      <span>{{ paramMeta(arg) }}</span>
    </div>
  </div>

  <div v-if="!recipe">
    <p class="desc">Choose a recipe from the Recipes tab.</p>
  </div>
  <template v-else>
    <ArgField
      v-for="arg in recipe.args"
      :key="arg.name"
      v-model="argValues[arg.name]"
      :arg="arg"
      @submit-preview="emit('submit-preview')"
    />
  </template>
</template>
