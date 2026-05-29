<script setup lang="ts">
import { ref, onMounted, nextTick } from "vue"

const emit = defineEmits<{
  confirm: [value: string]
  cancel: []
}>()

const model = defineModel<string>({ required: true })
const inputRef = ref<HTMLInputElement | null>(null)

onMounted(async () => {
  await nextTick()
  inputRef.value?.focus()
  inputRef.value?.select()
})
</script>

<template>
  <input
    ref="inputRef"
    v-model="model"
    class="tree-input"
    @keydown.enter="emit('confirm', model)"
    @keydown.escape="emit('cancel')"
    @keydown.stop
    @blur="emit('confirm', model)"
  />
</template>

<style scoped>
.tree-input {
  background: color-mix(in srgb, var(--bg) 72%, var(--surface));
  color: var(--fg);
  border: 1px solid var(--accent);
  padding: 3px 7px;
  font-size: 13px;
  font-family: inherit;
  border-radius: 5px;
  outline: none;
  width: 100%;
  box-shadow: 0 0 0 3px var(--accent-soft);
}
</style>
