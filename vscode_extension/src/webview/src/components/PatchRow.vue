<script setup lang="ts">
import type { LineChangeStats } from '../lib/diffStats';

defineProps<{
  label: string;
  stats: LineChangeStats;
  active: boolean;
  include: boolean;
  compact?: boolean;
}>();

const emit = defineEmits<{
  select: [];
  'update:include': [value: boolean];
}>();
</script>

<template>
  <div
    class="patch"
    :class="{ active, compact }"
    @click="emit('select')"
  >
    <input
      type="checkbox"
      :checked="include"
      class="patch-toggle"
      @click.stop
      @change="emit('update:include', ($event.target as HTMLInputElement).checked)"
    />
    <span class="patch-label">{{ label }}</span>
    <span class="diff-stats">
      <span v-if="stats.additions > 0" class="diff-stat-add">+{{ stats.additions }}</span>
      <span v-if="stats.deletions > 0" class="diff-stat-del">-{{ stats.deletions }}</span>
    </span>
  </div>
</template>
