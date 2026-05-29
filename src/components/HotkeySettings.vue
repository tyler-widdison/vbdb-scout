<script setup lang="ts">
import { ref } from "vue";
import {
  eventToHotkey,
  formatHotkey,
  useExplorerHotkeys,
  type ExplorerHotkeyAction,
} from "../composables/useExplorerHotkeys";

type HotkeyRow = {
  action: ExplorerHotkeyAction;
  label: string;
  description: string;
};

const rows: HotkeyRow[] = [
  {
    action: "videoPlayPause",
    label: "Play/Pause",
    description: "Toggle playback in video panel.",
  },
  {
    action: "videoSeekForward",
    label: "Fast forward",
    description: "Skip video forward by the configured seek step.",
  },
  {
    action: "videoSeekBackward",
    label: "Rewind",
    description: "Skip video backward by the configured seek step.",
  },
  {
    action: "videoSeekForwardOneMinute",
    label: "Seek forward one minute",
    description: "Skip video forward by 60 seconds.",
  },
  {
    action: "nextPlay",
    label: "Next play",
    description: "Move selection to next filtered play row.",
  },
  {
    action: "previousPlay",
    label: "Previous play",
    description: "Move selection to previous filtered play row.",
  },
  {
    action: "togglePlaySelection",
    label: "Add/Remove play",
    description: "Toggle active play in montage selection.",
  },
];

const { hotkeys, setHotkey, resetHotkey } = useExplorerHotkeys();
const recording = ref<ExplorerHotkeyAction | null>(null);

function capture(action: ExplorerHotkeyAction, e: KeyboardEvent) {
  e.preventDefault();
  e.stopPropagation();

  if (e.key === "Escape") {
    recording.value = null;
    return;
  }

  if (e.key === "Backspace" || e.key === "Delete") {
    setHotkey(action, null);
    recording.value = null;
    return;
  }

  const binding = eventToHotkey(e);
  if (!binding) return;

  setHotkey(action, binding);
  recording.value = null;
}
</script>

<template>
  <div class="hotkeys-panel">
    <div class="hotkeys-search">
      <div>
        <p class="search-title">Search hotkeys</p>
        <p class="search-subtitle">Showing {{ rows.length }} hotkeys.</p>
      </div>
      <div class="filter-box">
        <span class="filter-icon">&#x2315;</span>
        <span>Filter...</span>
      </div>
    </div>

    <div class="hotkey-list">
      <div v-for="row in rows" :key="row.action" class="hotkey-row">
        <div class="hotkey-copy">
          <p class="hotkey-title">{{ row.label }}</p>
          <p class="hotkey-description">{{ row.description }}</p>
        </div>
        <div class="hotkey-actions">
          <button
            class="binding-btn"
            :class="{ recording: recording === row.action }"
            @click="recording = row.action"
            @keydown="capture(row.action, $event)"
          >
            {{
              recording === row.action
                ? "Press keys..."
                : formatHotkey(hotkeys[row.action])
            }}
          </button>
          <button
            class="small-btn"
            title="Reset"
            @click="resetHotkey(row.action)"
          >
            &#x21BA;
          </button>
          <button
            class="small-btn"
            title="Clear"
            @click="setHotkey(row.action, null)"
          >
            &#x00D7;
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.hotkeys-panel {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.hotkeys-search {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 20px;
}

.search-title,
.hotkey-title {
  margin: 0;
  color: var(--fg);
  font-size: 15px;
}

.search-subtitle,
.hotkey-description {
  margin: 3px 0 0;
  color: var(--text-muted);
  font-size: 12px;
}

.filter-box {
  min-width: 190px;
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text-muted);
  background: color-mix(in srgb, var(--surface) 44%, transparent);
  border: 1px solid var(--border-soft);
  border-radius: 6px;
  padding: 7px 10px;
  font-size: 13px;
}

.filter-icon {
  font-size: 16px;
}

.hotkey-list {
  border-top: 1px solid var(--border-soft);
}

.hotkey-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 20px;
  min-height: 58px;
  border-bottom: 1px solid var(--border-soft);
}

.hotkey-copy {
  min-width: 0;
}

.hotkey-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.binding-btn,
.small-btn {
  border: 0;
  color: var(--fg);
  background: color-mix(in srgb, var(--surface) 72%, transparent);
  border-radius: 5px;
  cursor: pointer;
}

.binding-btn {
  min-width: 112px;
  padding: 4px 8px;
  font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
  font-size: 12px;
  font-weight: 700;
}

.binding-btn.recording {
  color: var(--accent);
  box-shadow: 0 0 0 2px var(--accent-soft);
}

.small-btn {
  width: 24px;
  height: 24px;
  color: var(--text-muted);
  font-size: 14px;
}

.binding-btn:hover,
.small-btn:hover {
  background: var(--surface-soft);
  color: var(--fg);
}
</style>
