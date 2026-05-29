<script setup lang="ts">
import { ref, watch } from "vue"

const props = defineProps<{
  open: boolean
  title: string
  label: string
  initialValue?: string
}>()

const emit = defineEmits<{
  close: []
  submit: [value: string]
}>()

const value = ref("")

watch(() => props.open, (isOpen) => {
  if (isOpen) value.value = props.initialValue ?? ""
})

function submit() {
  const next = value.value.trim()
  if (!next) return
  emit("submit", next)
}
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="overlay" @click.self="emit('close')">
      <div class="dialog" role="dialog" aria-modal="true">
        <h3>{{ title }}</h3>
        <label>
          <span>{{ label }}</span>
          <input v-model="value" @keydown.enter="submit" />
        </label>
        <div class="actions">
          <button class="secondary" @click="emit('close')">Cancel</button>
          <button class="primary" @click="submit">Save</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: grid;
  place-items: center;
  z-index: 250;
}

.dialog {
  width: min(420px, calc(100vw - 32px));
  border: 1px solid var(--border-soft);
  border-radius: 10px;
  background: color-mix(in srgb, var(--bg) 92%, #111);
  padding: 16px;
}

h3 {
  margin: 0 0 12px;
  font-size: 15px;
}

label span {
  display: block;
  color: var(--text-muted);
  font-size: 12px;
  margin-bottom: 6px;
}

input {
  width: 100%;
  border: 1px solid var(--border-soft);
  border-radius: 8px;
  background: color-mix(in srgb, var(--surface) 56%, transparent);
  color: var(--fg);
  padding: 9px 10px;
}

.actions {
  margin-top: 14px;
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

button {
  border-radius: 8px;
  border: 1px solid var(--border-soft);
  padding: 7px 10px;
  cursor: pointer;
}

.secondary {
  background: transparent;
  color: var(--fg);
}

.primary {
  background: var(--accent-soft);
  color: var(--accent);
  border-color: var(--accent-border);
}
</style>
