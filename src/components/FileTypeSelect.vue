<script setup lang="ts">
import { ref, onMounted } from "vue"
import { getFileType, setFileType } from "../services/api/settings"
import type { FileType } from "../types/database"

const options: FileType[] = ["json", "txt", "csv", "dvw"]
const selected = ref<FileType>("json")

onMounted(async () => {
  selected.value = (await getFileType()) as FileType
})

async function onChange() {
  await setFileType(selected.value)
}
</script>

<template>
  <div class="file-type-select-wrapper">
    <label for="file-type-select">File Type</label>
    <select id="file-type-select" v-model="selected" @change="onChange">
      <option v-for="opt in options" :key="opt" :value="opt">.{{ opt }}</option>
    </select>
  </div>
</template>

<style scoped>
.file-type-select-wrapper {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.file-type-select-wrapper label {
  font-size: 12px;
  color: var(--text-muted);
  font-weight: 650;
  letter-spacing: 0.04em;
  text-transform: uppercase;
}

.file-type-select-wrapper select {
  appearance: none;
  background: color-mix(in srgb, var(--bg) 72%, var(--surface));
  color: var(--fg);
  border: 1px solid var(--border-soft);
  padding: 10px 34px 10px 12px;
  border-radius: 10px;
  font-size: 14px;
  font-family: inherit;
  cursor: pointer;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%23888' d='M2 4l4 4 4-4'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 10px center;
  transition: border-color 140ms ease, box-shadow 140ms ease, background 140ms ease;
}

.file-type-select-wrapper select:hover {
  border-color: var(--accent-border);
}

.file-type-select-wrapper select:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 3px var(--accent-soft);
}
</style>
