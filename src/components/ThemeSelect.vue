<script setup lang="ts">
import { useTheme } from "../composables/useTheme"

const { currentName, themes } = useTheme()

function selectTheme(name: string) {
  currentName.value = name
}
</script>

<template>
  <div class="theme-grid">
    <button
      v-for="t in themes"
      :key="t.name"
      class="theme-card"
      :class="{ active: currentName === t.name }"
      @click="selectTheme(t.name)"
    >
      <div class="swatch-bar">
        <span class="swatch" :style="{ background: t.colors.bg }"></span>
        <span class="swatch" :style="{ background: t.colors.surface }"></span>
        <span class="swatch" :style="{ background: t.colors.accent }"></span>
        <span class="swatch" :style="{ background: t.colors.fg }"></span>
      </div>
      <span class="theme-label">{{ t.name }}</span>
    </button>
  </div>
</template>

<style scoped>
.theme-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.theme-card {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 10px;
  border: 1px solid var(--border-soft);
  border-radius: 10px;
  background: color-mix(in srgb, var(--surface) 42%, transparent);
  cursor: pointer;
  transition: border-color 140ms ease, box-shadow 140ms ease, background 140ms ease, transform 140ms ease;
  width: 120px;
}

.theme-card:hover {
  border-color: var(--accent-border);
  background: color-mix(in srgb, var(--surface) 64%, transparent);
  transform: translateY(-1px);
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.18);
}

.theme-card.active {
  border-color: var(--accent);
  background: var(--accent-soft);
  box-shadow: 0 0 0 2px var(--accent-soft), 0 6px 20px rgba(0, 0, 0, 0.22);
}

.swatch-bar {
  display: flex;
  gap: 3px;
  height: 32px;
  border-radius: 6px;
  overflow: hidden;
}

.swatch {
  flex: 1;
  border-radius: 4px;
}

.theme-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--fg);
  text-align: center;
  letter-spacing: 0.01em;
}

.theme-card.active .theme-label {
  color: var(--accent);
}
</style>
