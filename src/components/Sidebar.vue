<script setup lang="ts">
import { useSettings } from "../composables/useSettings";
import SettingsModal from "./SettingsModal.vue";

const { open, toggleSettings } = useSettings();
</script>

<template>
  <aside class="sidebar">
    <div class="sidebar-icons">
      <RouterLink to="/" class="icon-btn" title="Home">
        <span class="nav-icon">&#x2302;</span>
      </RouterLink>
      <RouterLink to="/seasons" class="icon-btn" title="Montage">
        <span class="nav-icon">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            height="20px"
            viewBox="0 -960 960 960"
            width="20px"
            fill="#e3e3e3"
          >
            <path
              d="M480-264q72 0 120-49t48-119q0-69-48-118.5T480-600q-72 0-120 49.5t-48 119q0 69.5 48 118.5t120 49Zm0-72q-42 0-69-28.13T384-433q0-39.9 27-67.45Q438-528 480-528t69 27.55q27 27.55 27 67.45 0 40.74-27 68.87Q522-336 480-336ZM168-144q-29 0-50.5-21.5T96-216v-432q0-29 21.5-50.5T168-720h120l72-96h240l72 96h120q29.7 0 50.85 21.5Q864-677 864-648v432q0 29-21.15 50.5T792-144H168Zm0-72h624v-432H636l-72.1-96H396l-72 96H168v432Zm312-217Z"
            /></svg
        ></span>
      </RouterLink>
      <div class="spacer"></div>
      <button
        class="icon-btn"
        :class="{ active: open }"
        @click="toggleSettings"
        title="Settings (Ctrl+,)"
      >
        &#x2699;&#xFE0E;
      </button>
    </div>
  </aside>
  <SettingsModal />
</template>

<style scoped>
.sidebar {
  display: flex;
  border-right: 1px solid var(--border-soft);
  background: color-mix(in srgb, var(--bg) 90%, var(--surface));
  flex-shrink: 0;
  box-shadow: inset -1px 0 0 color-mix(in srgb, var(--fg) 5%, transparent);
}

.sidebar-icons {
  width: 56px;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 12px 8px;
  gap: 8px;
  height: 100%;
}

.spacer {
  flex: 1;
}

.icon-btn {
  position: relative;
  background: transparent;
  border: 1px solid transparent;
  color: var(--fg);
  opacity: 0.68;
  width: 38px;
  height: 38px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  cursor: pointer;
  border-radius: 10px;
  text-decoration: none;
  transition:
    opacity 140ms ease,
    background 140ms ease,
    border-color 140ms ease,
    transform 140ms ease;
}

.icon-btn:hover {
  opacity: 1;
  background: var(--surface-soft);
  border-color: var(--border-soft);
  transform: translateY(-1px);
}

.icon-btn.active {
  opacity: 1;
  background: var(--accent-soft);
  border-color: var(--accent-border);
  color: var(--accent);
}

a.icon-btn.router-link-exact-active {
  opacity: 1;
  background: var(--accent-soft);
  border-color: var(--accent-border);
  color: var(--accent);
}

a.icon-btn.router-link-exact-active::before,
.icon-btn.active::before {
  content: "";
  position: absolute;
  left: -9px;
  width: 3px;
  height: 18px;
  border-radius: 999px;
  background: var(--accent);
}
</style>
