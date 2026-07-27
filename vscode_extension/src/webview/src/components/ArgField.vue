<script setup lang="ts">
import { computed } from 'vue';
import { ARG_INPUT_KIND, type RecipeArg } from '../shared';
import { effectiveInputKind } from '../lib/args';
import { useExtensionClient } from '../composables/useExtensionClient';

const client = useExtensionClient();

const props = defineProps<{
  arg: RecipeArg;
}>();

const model = defineModel<string>({ required: true });

const inputKind = computed(() => effectiveInputKind(props.arg));

const listId = computed(() => `options-${props.arg.name}`);

const isChecked = computed({
  get: () => model.value === 'true',
  set: (checked: boolean) => {
    model.value = checked ? 'true' : 'false';
  },
});

async function pickPath() {
  const directory = inputKind.value === ARG_INPUT_KIND.directory;
  const value = await client.pickPath(props.arg.name, directory);
  model.value = value;
}
</script>

<template>
  <label>
    {{ arg.name }}{{ arg.required ? ' *' : '' }}
    <span v-if="arg.help" class="help"> — {{ arg.help }}</span>
  </label>
  <div v-if="inputKind === ARG_INPUT_KIND.bool" class="row">
    <input
      :id="'arg-' + arg.name"
      v-model="isChecked"
      type="checkbox"
      @keydown.enter.prevent="$emit('submit-preview')"
    />
  </div>
  <div v-else class="row">
    <datalist v-if="arg.options?.length" :id="listId">
      <option v-for="opt in arg.options" :key="opt" :value="opt" />
    </datalist>
    <input
      :id="'arg-' + arg.name"
      v-model="model"
      type="text"
      :list="arg.options?.length ? listId : undefined"
      :placeholder="
        arg.options?.length && arg.allowCustomValue === false
          ? 'Choose one of the suggested values'
          : undefined
      "
      @keydown.enter.prevent="$emit('submit-preview')"
    />
    <button
      v-if="inputKind === ARG_INPUT_KIND.file || inputKind === ARG_INPUT_KIND.directory"
      type="button"
      class="pick-btn"
      @click="pickPath"
    >
      {{ inputKind === ARG_INPUT_KIND.directory ? 'Browse Folder…' : 'Browse…' }}
    </button>
  </div>
</template>
