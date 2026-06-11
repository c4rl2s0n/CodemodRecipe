<script setup lang="ts">
import { computed } from 'vue';
import { ARG_INPUT_KIND, WEBVIEW_TO_EXTENSION, type RecipeArg } from '../shared';
import { effectiveInputKind } from '../lib/args';
import { postToExtension } from '../vsCodeApi';

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

function pickPath() {
  const type =
    inputKind.value === ARG_INPUT_KIND.directory
      ? WEBVIEW_TO_EXTENSION.pickDirectory
      : WEBVIEW_TO_EXTENSION.pickFile;
  postToExtension({ type, arg: props.arg.name });
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
