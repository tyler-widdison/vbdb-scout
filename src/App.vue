<script setup lang="ts">
import { onBeforeUnmount, onMounted } from "vue";
import { useTheme } from "./composables/useTheme";
import { useSettings } from "./composables/useSettings";
import { useHotkey } from "./composables/useHotkey";
import { useRouter } from "vue-router";
import TitleBar from "./components/TitleBar.vue";
import Sidebar from "./components/Sidebar.vue";

useTheme();
const router = useRouter();
const { toggleSettings, closeSettings } = useSettings();

useHotkey({ key: ",", ctrl: true }, () => toggleSettings());
useHotkey({ key: "Escape" }, () => closeSettings());
useHotkey({ key: "E", ctrl: true, shift: true }, async () => {
  if (router.currentRoute.value.name !== "home")
    await router.push({ name: "home" });
  requestAnimationFrame(() =>
    window.dispatchEvent(new CustomEvent("focus-season-explorer")),
  );
});

const zoomLevel = { value: 1 };
function applyZoom() {
  document.documentElement.style.zoom = `${zoomLevel.value}`;
}
function onGlobalKeydown(e: KeyboardEvent) {
  if (!e.ctrlKey) return;
  if (e.key === "-") {
    e.preventDefault();
    zoomLevel.value = Math.max(
      0.6,
      Math.round((zoomLevel.value - 0.1) * 10) / 10,
    );
    applyZoom();
    return;
  }
  if (e.key === "=" || e.key === "+") {
    e.preventDefault();
    zoomLevel.value = Math.min(
      1.8,
      Math.round((zoomLevel.value + 0.1) * 10) / 10,
    );
    applyZoom();
    return;
  }
  if (e.key === "0") {
    e.preventDefault();
    zoomLevel.value = 1;
    applyZoom();
  }
}

onMounted(() => window.addEventListener("keydown", onGlobalKeydown));
onBeforeUnmount(() => window.removeEventListener("keydown", onGlobalKeydown));
</script>

<template>
  <TitleBar />
  <div class="layout">
    <Sidebar />
    <main class="content">
      <RouterView />
    </main>
  </div>
  <div class="grain" aria-hidden="true"></div>
</template>

<style>
@font-face {
  font-family: "Cascadia Mono";
  font-style: normal;
  font-weight: 200 700;
  font-display: swap;
  src: url("/fonts/cascadia-mono-latin-ext.woff2") format("woff2");
  unicode-range:
    U+0100-02BA, U+02BD-02C5, U+02C7-02CC, U+02CE-02D7, U+02DD-02FF, U+0304,
    U+0308, U+0329, U+1D00-1DBF, U+1E00-1E9F, U+1EF2-1EFF, U+2020, U+20A0-20AB,
    U+20AD-20C0, U+2113, U+2C60-2C7F, U+A720-A7FF;
}

@font-face {
  font-family: "Cascadia Mono";
  font-style: normal;
  font-weight: 200 700;
  font-display: swap;
  src: url("/fonts/cascadia-mono-latin.woff2") format("woff2");
  unicode-range:
    U+0000-00FF, U+0131, U+0152-0153, U+02BB-02BC, U+02C6, U+02DA, U+02DC,
    U+0304, U+0308, U+0329, U+2000-206F, U+20AC, U+2122, U+2191, U+2193, U+2212,
    U+2215, U+FEFF, U+FFFD;
}

@font-face {
  font-family: "Plus Jakarta Sans";
  font-style: normal;
  font-weight: 400 700;
  font-display: swap;
  src: url("/fonts/plus-jakarta-sans-latin-ext.woff2") format("woff2");
  unicode-range:
    U+0100-02BA, U+02BD-02C5, U+02C7-02CC, U+02CE-02D7, U+02DD-02FF, U+0304,
    U+0308, U+0329, U+1D00-1DBF, U+1E00-1E9F, U+1EF2-1EFF, U+2020, U+20A0-20AB,
    U+20AD-20C0, U+2113, U+2C60-2C7F, U+A720-A7FF;
}

@font-face {
  font-family: "Plus Jakarta Sans";
  font-style: normal;
  font-weight: 400 700;
  font-display: swap;
  src: url("/fonts/plus-jakarta-sans-latin.woff2") format("woff2");
  unicode-range:
    U+0000-00FF, U+0131, U+0152-0153, U+02BB-02BC, U+02C6, U+02DA, U+02DC,
    U+0304, U+0308, U+0329, U+2000-206F, U+20AC, U+2122, U+2191, U+2193, U+2212,
    U+2215, U+FEFF, U+FFFD;
}

:root {
  font-family: "Plus Jakarta Sans", system-ui, sans-serif;
  font-size: 14px;
  line-height: 1.5;
  background: var(--bg);
  color: var(--fg);
  color-scheme: dark;
  --surface-soft: color-mix(in srgb, var(--surface) 58%, var(--bg));
  --border-soft: color-mix(in srgb, var(--border) 70%, transparent);
  --text-muted: color-mix(in srgb, var(--fg) 62%, var(--bg));
  --accent-soft: color-mix(in srgb, var(--accent) 18%, transparent);
  --accent-border: color-mix(in srgb, var(--accent) 42%, var(--border));
  --shadow-lg: 0 24px 70px rgba(0, 0, 0, 0.46);
  --shadow-sm: 0 10px 28px rgba(0, 0, 0, 0.22);
  --grain-opacity: 0.04;
}

*,
*::before,
*::after {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  min-height: 100vh;
  background:
    radial-gradient(ellipse at 20% 0%, var(--glow), transparent 50%),
    radial-gradient(ellipse at 80% 100%, var(--glow), transparent 50%),
    linear-gradient(160deg, var(--gradient-a), var(--bg) 60%);
  color: var(--fg);
  position: relative;
}

.grain {
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 9999;
  opacity: var(--grain-opacity);
  background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.85' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)'/%3E%3C/svg%3E");
  background-repeat: repeat;
  background-size: 180px 180px;
}

button,
select {
  font: inherit;
}

button:focus-visible,
select:focus-visible,
a:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
}

::selection {
  background: var(--accent-soft);
  color: var(--accent);
}

::-webkit-scrollbar {
  width: 10px;
  height: 10px;
}

::-webkit-scrollbar-thumb {
  background: var(--border-soft);
  border: 3px solid transparent;
  border-radius: 999px;
  background-clip: padding-box;
}

::-webkit-scrollbar-thumb:hover {
  background-color: var(--accent-border);
}

.layout {
  display: flex;
  height: calc(100vh - 32px);
}

.content {
  flex: 1;
  overflow: auto;
}

h1,
h2,
h3 {
  font-family: "Cascadia Mono", monospace;
  font-optical-sizing: auto;
  font-weight: 650;
  letter-spacing: -0.02em;
}

h1 {
  font-size: 28px;
  margin-bottom: 16px;
}

h2 {
  font-size: 20px;
}

h3 {
  font-size: 17px;
}

@keyframes reveal {
  from {
    opacity: 0;
    transform: translateY(6px);
  }
  to {
    opacity: 1;
    transform: none;
  }
}

.titlebar {
  animation: reveal 0.4s ease both;
}

.sidebar {
  animation: reveal 0.4s 0.06s ease both;
}

.content {
  animation: reveal 0.4s 0.12s ease both;
}
</style>
