<script setup lang="ts">
import { computed } from 'vue';
import type { RecipeArg, RecipeSchema } from '../shared';
import {
  slotBadgeLabel,
  slotBadgeTitle,
  slotsForRecipe,
} from '../lib/slotBadges';

const props = defineProps<{
  recipe: RecipeSchema;
  slotsByRecipe: Record<string, string[]>;
}>();

const slots = computed(() =>
  slotsForRecipe(props.slotsByRecipe, props.recipe.id)
);

const explorerMenu = computed(() => props.recipe.explorerMenu ?? []);
const postExecution = computed(() => props.recipe.postExecution ?? []);

const contextArgs = computed(() =>
  props.recipe.args.filter(
    (arg) => arg.from != null || (arg.contextKey != null && arg.contextKey !== '')
  )
);

function formatFrom(arg: RecipeArg): string {
  if (arg.from == null) {
    return arg.contextKey ? `contextKey=${arg.contextKey}` : '';
  }
  if (typeof arg.from === 'string') {
    return arg.from;
  }
  if ('template' in arg.from && arg.from.template) {
    return `template`;
  }
  if ('query' in arg.from) {
    return `query${arg.from.capture ? `#${arg.from.capture}` : ''}`;
  }
  return 'from';
}

function inputKindLabel(arg: RecipeArg): string {
  const kind = arg.inputKind;
  if (Array.isArray(kind)) {
    return `list<${kind[0] ?? '?'}>`;
  }
  return kind || 'text';
}

const hasInfo = computed(
  () =>
    slots.value.length > 0 ||
    explorerMenu.value.length > 0 ||
    postExecution.value.length > 0 ||
    contextArgs.value.length > 0
);
</script>

<template>
  <section v-if="hasInfo" class="recipe-info">
    <h3>Recipe info</h3>

    <div v-if="slots.length" class="info-block">
      <div class="info-label">Shortcuts</div>
      <div class="chips">
        <span
          v-for="slot in slots"
          :key="slot"
          class="chip slot"
          :title="slotBadgeTitle(slot)"
        >
          {{ slotBadgeLabel(slot) }}
        </span>
      </div>
    </div>

    <div v-if="contextArgs.length" class="info-block">
      <div class="info-label">Context args</div>
      <ul class="info-list">
        <li v-for="arg in contextArgs" :key="arg.name">
          <code>{{ arg.name }}</code>
          <span v-if="arg.required" class="chip">required</span>
          <span class="chip">{{ inputKindLabel(arg) }}</span>
          <span class="muted">{{ formatFrom(arg) }}</span>
        </li>
      </ul>
    </div>

    <div v-if="explorerMenu.length" class="info-block">
      <div class="info-label">Explorer menu</div>
      <ul class="info-list">
        <li v-for="(entry, index) in explorerMenu" :key="index">
          <span class="chip">{{ entry.kind }}</span>
          <span v-if="entry.if" class="muted">if {{ entry.if }}</span>
          <span
            v-if="entry.args && Object.keys(entry.args).length"
            class="muted"
          >
            args: {{ Object.keys(entry.args).join(', ') }}
          </span>
        </li>
      </ul>
    </div>

    <div v-if="postExecution.length" class="info-block">
      <div class="info-label">Post-execution</div>
      <ul class="info-list">
        <li v-for="(cmd, index) in postExecution" :key="index">
          <code>{{ cmd }}</code>
        </li>
      </ul>
    </div>
  </section>
</template>

<style scoped>
.recipe-info {
  margin: 12px 0 16px;
  font-size: 12px;
}
.info-block {
  margin-top: 8px;
}
.info-label {
  font-weight: 600;
  margin-bottom: 4px;
  opacity: 0.9;
}
.info-list {
  margin: 0;
  padding-left: 1.2em;
}
.info-list li {
  margin: 2px 0;
}
.chips {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}
.chip {
  display: inline-block;
  padding: 0 5px;
  border-radius: 3px;
  background: var(--vscode-badge-background);
  color: var(--vscode-badge-foreground);
  font-size: 11px;
  margin-right: 4px;
}
.chip.slot {
  font-family: var(--vscode-editor-font-family, monospace);
}
.muted {
  opacity: 0.75;
  margin-left: 4px;
}
</style>
