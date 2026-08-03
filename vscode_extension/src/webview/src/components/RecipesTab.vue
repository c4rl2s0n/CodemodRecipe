<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import type {
  ContextRecipeMatch,
  RecipeDiagnostic,
  RecipeSchema,
} from '../shared';
import { useExtensionClient } from '../composables/useExtensionClient';
import RecipeGroupNode from './RecipeGroupNode.vue';
import {
  buildRecipeTree,
  recipeDisplayTitle,
  type RecipeTreeNode,
} from '../lib/recipeTree';
import { slotBadgeLabel, slotBadgeTitle, slotsForRecipe } from '../lib/slotBadges';

const client = useExtensionClient();

const props = defineProps<{
  recipes: readonly RecipeSchema[];
  discoveryError?: string;
  diagnostics: readonly RecipeDiagnostic[];
  refreshing: boolean;
  contextMatches: readonly ContextRecipeMatch[];
  slotsByRecipe: Record<string, string[]>;
}>();

const searchQuery = ref('');
const collapsedGroups = ref<Record<string, boolean>>({});
const contextExpanded = ref(true);

type RecipeContextMenuState = {
  recipeId: string;
  args?: Record<string, string>;
  x: number;
  y: number;
};

const recipeContextMenu = ref<RecipeContextMenuState | null>(null);

const errorDiagnostics = computed(() =>
  props.diagnostics.filter((item) => item.severity === 'error')
);
const warningDiagnostics = computed(() =>
  props.diagnostics.filter((item) => item.severity === 'warning')
);

const filteredRecipes = computed(() => {
  const q = searchQuery.value.trim().toLowerCase();
  if (!q) {
    return props.recipes;
  }
  return props.recipes.filter((recipe) => {
    const haystack = [
      recipe.id,
      recipe.name,
      recipe.description,
      recipeDisplayTitle(recipe),
    ]
      .join(' ')
      .toLowerCase();
    return haystack.includes(q);
  });
});

const recipeTree = computed((): RecipeTreeNode[] => {
  return buildRecipeTree(filteredRecipes.value);
});

function countRecipes(node: RecipeTreeNode): number {
  return (
    node.recipes.length +
    node.children.reduce((sum, child) => sum + countRecipes(child), 0)
  );
}

function isCollapsed(key: string): boolean {
  return collapsedGroups.value[key] !== false;
}

function toggleGroup(key: string) {
  collapsedGroups.value = {
    ...collapsedGroups.value,
    [key]: !isCollapsed(key),
  };
}

function selectRecipe(id: string) {
  client.selectRecipe(id);
}

function closeRecipeContextMenu(): void {
  recipeContextMenu.value = null;
}

function onRecipeContextMenu(recipe: RecipeSchema, event: MouseEvent): void {
  event.preventDefault();
  recipeContextMenu.value = {
    recipeId: recipe.id,
    x: event.clientX,
    y: event.clientY,
  };
}

function onContextMatchMenu(match: ContextRecipeMatch, event: MouseEvent): void {
  event.preventDefault();
  recipeContextMenu.value = {
    recipeId: match.recipeId,
    args: match.args,
    x: event.clientX,
    y: event.clientY,
  };
}

function showRecipeFromContextMenu(): void {
  const id = recipeContextMenu.value?.recipeId;
  if (id) {
    client.openRecipeFile(id);
  }
  closeRecipeContextMenu();
}

function createShortcutFromContextMenu(): void {
  const menu = recipeContextMenu.value;
  if (menu) {
    client.createShortcut(menu.recipeId, menu.args);
  }
  closeRecipeContextMenu();
}

function runContextMatch(match: ContextRecipeMatch, mode: 'auto' | 'open'): void {
  client.invokeRecipe(match.recipeId, mode, match.args);
}

onMounted(() => {
  window.addEventListener('click', closeRecipeContextMenu);
  window.addEventListener('scroll', closeRecipeContextMenu, true);
});

onUnmounted(() => {
  window.removeEventListener('click', closeRecipeContextMenu);
  window.removeEventListener('scroll', closeRecipeContextMenu, true);
});

function refresh() {
  client.refreshRecipes();
}

function configureHost() {
  client.configureHost();
}

function scaffoldProject() {
  client.scaffoldProject();
}

function recipeSubtitle(recipe: RecipeSchema): string {
  return (
    recipe.description ||
    recipe.args.map((arg) => arg.name).join(', ')
  );
}

function recipeTitle(recipe: RecipeSchema): string {
  return recipeDisplayTitle(recipe);
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

function formatArgs(args: Record<string, string>): string {
  return Object.entries(args)
    .map(([k, v]) => `${k}: ${v}`)
    .join(', ');
}
</script>

<template>
  <div>
    <h2>Recipes</h2>

    <div class="context-expander">
      <button
        type="button"
        class="group-toggle secondary context-toggle"
        @click="contextExpanded = !contextExpanded"
      >
        <span class="group-chevron">{{ contextExpanded ? '▾' : '▸' }}</span>
        <span class="group-label">Context</span>
        <span class="group-count">{{ contextMatches.length }}</span>
      </button>
      <div v-if="contextExpanded" class="context-body">
        <p v-if="!contextMatches.length" class="desc context-empty">
          No recipes match the current editor context. Open a file and place the
          cursor where recipe <code>from</code> builtins apply.
        </p>
        <div v-else class="recipe-list context-list">
          <div
            v-for="match in contextMatches"
            :key="match.recipeId"
            class="recipe-row context-row"
            @contextmenu="onContextMatchMenu(match, $event)"
          >
            <button
              type="button"
              class="recipe-button secondary"
              @click="runContextMatch(match, 'open')"
            >
              <span class="recipe-title-row">
                <span class="recipe-title">{{ match.name }}</span>
                <span
                  v-if="match.complete"
                  class="badge badge-ready"
                  title="All required args filled"
                >ready</span>
                <span
                  v-else
                  class="badge badge-partial"
                  title="Some required args still missing"
                >partial</span>
                <span
                  v-for="slot in slotsForRecipe(slotsByRecipe, match.recipeId)"
                  :key="slot"
                  class="badge badge-slot"
                  :title="slotBadgeTitle(slot)"
                >{{ slotBadgeLabel(slot) }}</span>
              </span>
              <span class="recipe-group-path">{{ match.recipeId }}</span>
              <span class="recipe-desc">{{ formatArgs(match.args) }}</span>
            </button>
            <div class="context-actions">
              <button
                type="button"
                class="secondary context-action"
                title="Apply when complete"
                @click.stop="runContextMatch(match, 'auto')"
              >
                Run
              </button>
              <button
                type="button"
                class="secondary context-action"
                title="Open in Recipe Runner"
                @click.stop="runContextMatch(match, 'open')"
              >
                Open
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

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
          Set Codemod Root Directory
        </button>
        <button type="button" class="secondary" @click="scaffoldProject">
          Scaffold .codemod
        </button>
      </div>
    </div>
    <div v-else class="recipe-browser">
      <input
        v-model="searchQuery"
        type="text"
        class="recipe-search"
        placeholder="Search recipes…"
        aria-label="Search recipes"
      />
      <div v-if="!filteredRecipes.length" class="empty-state">
        <p class="desc">No recipes match “{{ searchQuery }}”.</p>
      </div>
      <div v-else class="recipe-tree">
        <RecipeGroupNode
          v-for="node in recipeTree"
          :key="node.key"
          :node="node"
          :depth="0"
          :is-collapsed="isCollapsed"
          :toggle-group="toggleGroup"
          :count-recipes="countRecipes"
          :select-recipe="selectRecipe"
          :on-recipe-context-menu="onRecipeContextMenu"
          :recipe-subtitle="recipeSubtitle"
          :recipe-title="recipeTitle"
          :slots-by-recipe="slotsByRecipe"
        />
      </div>
    </div>

    <Teleport to="body">
      <div
        v-if="recipeContextMenu"
        class="recipe-context-menu"
        :style="{
          left: `${recipeContextMenu.x}px`,
          top: `${recipeContextMenu.y}px`,
        }"
        role="menu"
        @click.stop
      >
        <button
          type="button"
          class="recipe-context-menu-item"
          role="menuitem"
          @click="showRecipeFromContextMenu"
        >
          Show Recipe
        </button>
        <button
          type="button"
          class="recipe-context-menu-item"
          role="menuitem"
          @click="createShortcutFromContextMenu"
        >
          Create shortcut…
        </button>
      </div>
    </Teleport>
  </div>
</template>
