<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";

const appWindow = getCurrentWindow();

let lastClick = 0;

function handleTitleMouseDown() {
  const now = Date.now();
  if (now - lastClick < 400) {
    appWindow.toggleMaximize();
    lastClick = 0;
    return;
  }
  lastClick = now;
  appWindow.startDragging();
}

function minimize() {
  appWindow.minimize();
}

function toggleMaximize() {
  appWindow.toggleMaximize();
}

function close() {
  appWindow.close();
}
</script>

<template>
  <div class="titlebar" @mousedown="handleTitleMouseDown">
    <span class="title">vbdb-scout</span>
    <div class="controls" @mousedown.stop>
      <button @click="minimize" title="Minimize">&#x2500;</button>
      <button @click="toggleMaximize" title="Maximize">&#x25A1;</button>
      <button class="close" @click="close" title="Close">&#x00D7;</button>
    </div>
  </div>
</template>

<style scoped>
.titlebar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  height: 32px;
  background: color-mix(in srgb, var(--bg) 82%, transparent);
  color: var(--fg);
  padding-left: 12px;
  user-select: none;
  border-bottom: 1px solid var(--border-soft);
  backdrop-filter: blur(18px);
}

.title {
  color: var(--text-muted);
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.02em;
}

.controls {
  display: flex;
  height: 100%;
}

.controls button {
  background: none;
  border: none;
  color: var(--fg);
  width: 46px;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  cursor: pointer;
  transition:
    background 140ms ease,
    color 140ms ease;
}

.controls button:hover {
  background: var(--surface-soft);
}

.controls button.close:hover {
  background: #e81123;
  color: #fff;
}
</style>
