<script setup lang="ts">
import { ref } from 'vue';
import type { CoverageTreeNode } from '../lib/coverageTree';

const props = defineProps<{
  nodes: readonly CoverageTreeNode[];
  depth?: number;
}>();

const emit = defineEmits<{
  openFile: [node: CoverageTreeNode];
}>();

const expanded = ref<Record<string, boolean>>({});

function toggle(path: string): void {
  expanded.value = {
    ...expanded.value,
    [path]: !expanded.value[path],
  };
}

function isExpanded(node: CoverageTreeNode): boolean {
  if (expanded.value[node.path] !== undefined) {
    return expanded.value[node.path]!;
  }
  return (props.depth ?? 0) < 2;
}

function pct(ratio: number | undefined): string {
  if (ratio === undefined || Number.isNaN(ratio)) {
    return '';
  }
  return `${Math.round(ratio * 100)}%`;
}

function onOpen(node: CoverageTreeNode): void {
  emit('openFile', node);
}
</script>

<template>
  <ul class="cov-tree" :style="{ '--depth': depth ?? 0 }">
    <li v-for="node in nodes" :key="node.path || node.name">
      <div
        class="cov-row"
        :class="[`tint-${node.tint}`, { file: node.kind === 'file' }]"
      >
        <button
          v-if="node.kind === 'dir'"
          type="button"
          class="expand"
          @click="toggle(node.path)"
        >
          {{ isExpanded(node) ? '▾' : '▸' }}
        </button>
        <span v-else class="expand-spacer" />
        <button
          v-if="node.kind === 'file'"
          type="button"
          class="cov-label"
          @click="onOpen(node)"
        >
          <span class="dot" :class="`dot-${node.tint}`" />
          <span class="name">{{ node.name }}</span>
          <span class="pct">{{ pct(node.ratio) }}</span>
        </button>
        <div v-else class="cov-label dir">
          <span class="dot" :class="`dot-${node.tint}`" />
          <span class="name">{{ node.name }}</span>
        </div>
      </div>
      <CoverageTree
        v-if="node.kind === 'dir' && isExpanded(node) && node.children?.length"
        :nodes="node.children"
        :depth="(depth ?? 0) + 1"
        @open-file="onOpen"
      />
    </li>
  </ul>
</template>

<script lang="ts">
export default {
  name: 'CoverageTree',
};
</script>

<style scoped>
.cov-tree {
  list-style: none;
  padding: 0;
  margin: 0;
  padding-left: calc(var(--depth, 0) * 10px);
}
.cov-row {
  display: flex;
  align-items: center;
  gap: 2px;
}
.expand,
.cov-label {
  background: transparent;
  border: none;
  color: inherit;
  cursor: pointer;
  padding: 3px 2px;
  text-align: left;
}
.expand-spacer {
  width: 16px;
  flex-shrink: 0;
}
.cov-label {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
  min-width: 0;
}
.cov-label.dir {
  cursor: default;
}
.name {
  font-family: var(--vscode-editor-font-family, monospace);
  font-size: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.pct {
  font-size: 11px;
  opacity: 0.75;
  margin-left: auto;
}
.dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}
.dot-green {
  background: var(--cov-green, #3d9a5f);
}
.dot-yellow {
  background: var(--cov-yellow, #c4a035);
}
.dot-red {
  background: var(--cov-red, #c44b4b);
}
.cov-label.file:hover .name {
  text-decoration: underline;
}
</style>
