<script setup lang="ts">
import { computed } from 'vue';
import type { PatchInfo } from '../types';

const props = defineProps<{
  patch: PatchInfo;
  active: boolean;
  include: boolean;
  isWholeFile: boolean;
}>();

const emit = defineEmits<{
  select: [];
  'update:include': [value: boolean];
}>();

const replacementText = computed(() => {
  const text =
    props.patch.replacement || props.patch.replacementPreview || '';
  const desc = props.patch.description ? props.patch.description + '\n' : '';
  return desc + text.trim();
});
</script>

<template>
  <div
    class="patch"
    :class="{ active }"
    @click="emit('select')"
  >
    <input
      type="checkbox"
      :checked="include"
      :class="isWholeFile ? 'whole-file-toggle' : 'patch-toggle'"
      @click.stop
      @change="emit('update:include', ($event.target as HTMLInputElement).checked)"
    />
    <code>{{ replacementText }}</code>
  </div>
</template>
