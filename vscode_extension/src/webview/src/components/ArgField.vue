<script setup lang="ts">
import { computed, ref } from 'vue';
import { ARG_INPUT_KIND, type ArgValue, type RecipeArg } from '../shared';
import {
  asList,
  asObject,
  effectiveInputKind,
  emptyValue,
  isListKind,
  isNamedType,
  itemKind,
  rowSummary,
} from '../lib/args';
import { useExtensionClient } from '../composables/useExtensionClient';

defineOptions({ name: 'ArgField' });

const client = useExtensionClient();

const props = defineProps<{
  arg: RecipeArg;
  hideLabel?: boolean;
  nested?: boolean;
}>();

const model = defineModel<ArgValue>({ required: true });

const emit = defineEmits<{
  'submit-preview': [];
}>();

const expandedIndex = ref<number | null>(null);

const inputKind = computed(() => effectiveInputKind(props.arg));
const list = computed(() => isListKind(props.arg.inputKind));
const named = computed(() => isNamedType(itemKind(props.arg.inputKind)));
const closedChoice = computed(
  () => inputKind.value === ARG_INPUT_KIND.choice
);
const listId = computed(() => `options-${props.arg.name}`);

const isChecked = computed({
  get: () => model.value === true || model.value === 'true',
  set: (checked: boolean) => {
    model.value = checked;
  },
});

const textModel = computed({
  get: () => (typeof model.value === 'string' ? model.value : ''),
  set: (value: string) => {
    model.value = value;
  },
});

const objectModel = computed({
  get: () => asObject(model.value),
  set: (value: Record<string, ArgValue>) => {
    model.value = value;
  },
});

const listModel = computed({
  get: () => asList(model.value),
  set: (value: ArgValue[]) => {
    model.value = value;
  },
});

const itemArg = computed((): RecipeArg => {
  const item = itemKind(props.arg.inputKind);
  return {
    ...props.arg,
    name: 'item',
    required: true,
    inputKind: item,
    fields: props.arg.fields,
  };
});

function setObjectField(name: string, value: ArgValue) {
  objectModel.value = { ...objectModel.value, [name]: value };
}

function setListItem(index: number, value: ArgValue) {
  const next = [...listModel.value];
  next[index] = value;
  listModel.value = next;
}

function addRow() {
  const next = [...listModel.value, emptyValue(itemArg.value)];
  listModel.value = next;
  expandedIndex.value = next.length - 1;
}

function removeRow(index: number) {
  const next = listModel.value.filter((_, i) => i !== index);
  listModel.value = next;
  if (expandedIndex.value === index) {
    expandedIndex.value = next.length ? next.length - 1 : null;
  } else if (expandedIndex.value !== null && expandedIndex.value > index) {
    expandedIndex.value -= 1;
  }
}

function toggleRow(index: number) {
  expandedIndex.value = expandedIndex.value === index ? null : index;
}

async function pickPath() {
  const directory = inputKind.value === ARG_INPUT_KIND.directory;
  const value = await client.pickPath(props.arg.name, directory);
  model.value = value;
}
</script>

<template>
  <div :class="['arg-field', { nested: nested }]">
    <template v-if="list">
      <div class="list-header">
        <label v-if="!hideLabel">
          {{ arg.name }}{{ arg.required ? ' *' : '' }}
          <span v-if="arg.help" class="help"> — {{ arg.help }}</span>
        </label>
        <button type="button" class="icon-btn" @click="addRow">Add</button>
      </div>
      <div
        v-for="(row, index) in listModel"
        :key="index"
        class="list-row"
      >
        <template v-if="named">
          <div class="row-toolbar">
            <button
              type="button"
              class="summary"
              @click="toggleRow(index)"
            >
              {{ rowSummary(arg.fields, row) }}
            </button>
            <button type="button" class="icon-btn" @click="removeRow(index)">
              Remove
            </button>
          </div>
          <div v-if="expandedIndex === index" class="object-fields nested">
            <ArgField
              v-for="field in arg.fields ?? []"
              :key="field.name"
              :arg="field"
              :model-value="asObject(row)[field.name] ?? emptyValue(field)"
              nested
              @update:model-value="
                setListItem(index, {
                  ...asObject(row),
                  [field.name]: $event,
                })
              "
              @submit-preview="emit('submit-preview')"
            />
          </div>
        </template>
        <div v-else class="row-toolbar">
          <ArgField
            :arg="itemArg"
            hide-label
            nested
            :model-value="row"
            @update:model-value="setListItem(index, $event)"
            @submit-preview="emit('submit-preview')"
          />
          <button type="button" class="icon-btn" @click="removeRow(index)">
            Remove
          </button>
        </div>
      </div>
    </template>

    <template v-else-if="named">
      <label v-if="!hideLabel">
        {{ arg.name }}{{ arg.required ? ' *' : '' }}
        <span v-if="arg.help" class="help"> — {{ arg.help }}</span>
      </label>
      <div class="object-fields nested">
        <ArgField
          v-for="field in arg.fields ?? []"
          :key="field.name"
          :arg="field"
          nested
          :model-value="objectModel[field.name] ?? emptyValue(field)"
          @update:model-value="setObjectField(field.name, $event)"
          @submit-preview="emit('submit-preview')"
        />
      </div>
    </template>

    <template v-else>
      <label v-if="!hideLabel">
        {{ arg.name }}{{ arg.required ? ' *' : '' }}
        <span v-if="arg.help" class="help"> — {{ arg.help }}</span>
        <span v-if="arg.fromRecipes?.length" class="nested-hint">
          Used by nested: {{ arg.fromRecipes.join(', ') }}
        </span>
      </label>
      <div v-if="inputKind === ARG_INPUT_KIND.bool" class="row">
        <input
          :id="'arg-' + arg.name"
          v-model="isChecked"
          type="checkbox"
          @keydown.enter.prevent="emit('submit-preview')"
        />
      </div>
      <div v-else-if="closedChoice" class="row">
        <select
          :id="'arg-' + arg.name"
          v-model="textModel"
          @keydown.enter.prevent="emit('submit-preview')"
        >
          <option v-if="!arg.required" value=""></option>
          <option v-for="opt in arg.options" :key="opt" :value="opt">
            {{ opt }}
          </option>
        </select>
      </div>
      <div v-else class="row">
        <datalist v-if="arg.options?.length" :id="listId">
          <option v-for="opt in arg.options" :key="opt" :value="opt" />
        </datalist>
        <input
          :id="'arg-' + arg.name"
          v-model="textModel"
          type="text"
          :list="arg.options?.length ? listId : undefined"
          @keydown.enter.prevent="emit('submit-preview')"
        />
        <button
          v-if="
            inputKind === ARG_INPUT_KIND.file ||
            inputKind === ARG_INPUT_KIND.directory
          "
          type="button"
          class="pick-btn"
          @click="pickPath"
        >
          {{
            inputKind === ARG_INPUT_KIND.directory
              ? 'Browse Folder…'
              : 'Browse…'
          }}
        </button>
      </div>
    </template>
  </div>
</template>

<style scoped>
.arg-field.nested {
  margin-left: 12px;
  padding-left: 8px;
  border-left: 1px solid var(--vscode-panel-border);
}

.list-header,
.row-toolbar {
  align-items: center;
  display: flex;
  gap: 8px;
  justify-content: space-between;
}

.list-header label {
  margin-top: 12px;
}

.list-row {
  margin-top: 8px;
}

.summary {
  background: transparent;
  color: inherit;
  flex: 1;
  margin-top: 0;
  text-align: left;
}

.icon-btn {
  margin-top: 0;
  padding: 4px 8px;
}

.object-fields {
  margin-top: 4px;
}

select {
  background: var(--vscode-input-background);
  border: 1px solid var(--vscode-input-border);
  color: var(--vscode-input-foreground);
  flex: 1;
  padding: 6px 8px;
}
</style>
