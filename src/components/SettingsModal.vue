<script setup lang="ts">
import { nextTick, onMounted, ref, watch } from "vue";
import { useSettings } from "../composables/useSettings";
import ThemeSelect from "./ThemeSelect.vue";
import HotkeySettings from "./HotkeySettings.vue";
import * as api from "../services/api";

const { open, activeTab, closeSettings } = useSettings();
const navEl = ref<HTMLElement | null>(null);
const contentEl = ref<HTMLElement | null>(null);
const autoSeason = ref(true);
const autoPlayOnSeek = ref(true);
const autoAdvanceMontage = ref(true);
const videoSeekStepSeconds = ref(3);
const muteOnAutoplayStart = ref(false);
const explorerListStyle = ref<"file" | "card">("file");
const autoSelectFilteredPlays = ref(false);
const showPrependingDate = ref(false);
const codeEditSaveMode = ref<"after_edit" | "end_of_editing">("after_edit");
const storedScoutFilesPath = ref("");
const scoutVisibleColumns = ref({ videoTime: true, set: true, score: true });

const settingsTabs = ["general", "settings", "theme", "hotkeys"] as const;

onMounted(async () => {
  await loadAutoSeasonSetting();
  await loadStoredScoutFilesPath();
  const stored = localStorage.getItem("autoPlayOnSeek");
  autoPlayOnSeek.value = stored !== null ? stored === "true" : true;
  const autoAdvanceStored = localStorage.getItem("autoAdvanceMontage");
  autoAdvanceMontage.value =
    autoAdvanceStored !== null ? autoAdvanceStored === "true" : true;
  const muteStored = localStorage.getItem("muteOnAutoplayStart");
  muteOnAutoplayStart.value =
    muteStored !== null ? muteStored === "true" : false;
  const seekStepStored = localStorage.getItem("videoSeekStepSeconds");
  const seekStep = Number(seekStepStored);
  videoSeekStepSeconds.value =
    Number.isFinite(seekStep) && seekStep > 0 ? seekStep : 3;
  explorerListStyle.value =
    localStorage.getItem("explorerListStyle") === "card" ? "card" : "file";
  autoSelectFilteredPlays.value =
    localStorage.getItem("autoSelectFilteredPlays") === "true";
  showPrependingDate.value =
    localStorage.getItem("showPrependingDate") === "true";
  codeEditSaveMode.value =
    localStorage.getItem("codeEditSaveMode") === "end_of_editing"
      ? "end_of_editing"
      : "after_edit";
  loadScoutVisibleColumns();
});

function loadScoutVisibleColumns() {
  const stored = localStorage.getItem("scoutVisibleColumns");
  if (!stored) return;
  try {
    const parsed = JSON.parse(stored) as Partial<
      typeof scoutVisibleColumns.value
    >;
    scoutVisibleColumns.value = {
      videoTime: parsed.videoTime !== false,
      set: parsed.set !== false,
      score: parsed.score !== false,
    };
  } catch {
    scoutVisibleColumns.value = { videoTime: true, set: true, score: true };
  }
}

async function loadAutoSeasonSetting() {
  try {
    autoSeason.value = await api.getAutoSeason();
  } catch {
    setTimeout(async () => {
      try {
        autoSeason.value = await api.getAutoSeason();
      } catch {
        autoSeason.value = true;
      }
    }, 300);
  }
}

async function loadStoredScoutFilesPath() {
  try {
    storedScoutFilesPath.value = await api.getStoredScoutFilesPath();
  } catch {
    storedScoutFilesPath.value = "Unavailable until app data is initialized";
  }
}

async function onAutoSeasonToggle(value: boolean) {
  autoSeason.value = value;
  try {
    await api.setAutoSeason(value);
  } catch {
    autoSeason.value = !value;
  }
}

function onAutoPlayOnSeekToggle(value: boolean) {
  autoPlayOnSeek.value = value;
  localStorage.setItem("autoPlayOnSeek", String(value));
  window.dispatchEvent(new Event("vbdb-settings-changed"));
}

function onAutoAdvanceMontageToggle(value: boolean) {
  autoAdvanceMontage.value = value;
  localStorage.setItem("autoAdvanceMontage", String(value));
  window.dispatchEvent(new Event("vbdb-settings-changed"));
}

function onMuteOnAutoplayStartToggle(value: boolean) {
  muteOnAutoplayStart.value = value;
  localStorage.setItem("muteOnAutoplayStart", String(value));
  window.dispatchEvent(new Event("vbdb-settings-changed"));
}

function onVideoSeekStepChange(value: string) {
  const parsed = Number(value);
  const safe = Number.isFinite(parsed) && parsed > 0 ? parsed : 3;
  videoSeekStepSeconds.value = safe;
  localStorage.setItem("videoSeekStepSeconds", String(safe));
  window.dispatchEvent(new Event("vbdb-settings-changed"));
}

function onExplorerListStyleChange(value: string) {
  const safe = value === "card" ? "card" : "file";
  explorerListStyle.value = safe;
  localStorage.setItem("explorerListStyle", safe);
  window.dispatchEvent(new Event("vbdb-settings-changed"));
}

function onAutoSelectFilteredPlaysToggle(value: boolean) {
  autoSelectFilteredPlays.value = value;
  localStorage.setItem("autoSelectFilteredPlays", String(value));
  window.dispatchEvent(new Event("vbdb-settings-changed"));
}

function onShowPrependingDateToggle(value: boolean) {
  showPrependingDate.value = value;
  localStorage.setItem("showPrependingDate", String(value));
  window.dispatchEvent(new Event("vbdb-settings-changed"));
}

function onCodeEditSaveModeChange(value: string) {
  const safe = value === "end_of_editing" ? "end_of_editing" : "after_edit";
  codeEditSaveMode.value = safe;
  localStorage.setItem("codeEditSaveMode", safe);
  window.dispatchEvent(new Event("vbdb-settings-changed"));
}

function onScoutColumnToggle(
  column: keyof typeof scoutVisibleColumns.value,
  value: boolean,
) {
  scoutVisibleColumns.value = { ...scoutVisibleColumns.value, [column]: value };
  localStorage.setItem(
    "scoutVisibleColumns",
    JSON.stringify(scoutVisibleColumns.value),
  );
  window.dispatchEvent(new Event("vbdb-settings-changed"));
}

async function copyStoredScoutFilesPath() {
  if (!storedScoutFilesPath.value) return;
  await navigator.clipboard.writeText(storedScoutFilesPath.value);
}

function onOverlayClick(e: MouseEvent) {
  if ((e.target as HTMLElement).classList.contains("overlay")) closeSettings();
}

function focusSettingsNav() {
  navEl.value?.focus();
}

function focusSettingsContent() {
  const selector =
    "button, select, input, textarea, [tabindex]:not([tabindex='-1'])";
  const control = contentEl.value?.querySelector<HTMLElement>(selector);
  control?.focus();
}

function onNavKeydown(e: KeyboardEvent) {
  if (e.key === "ArrowRight") {
    e.preventDefault();
    focusSettingsContent();
    return;
  }

  if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;

  e.preventDefault();
  const currentIndex = settingsTabs.indexOf(activeTab.value);
  const direction = e.key === "ArrowDown" ? 1 : -1;
  const nextIndex =
    (currentIndex + direction + settingsTabs.length) % settingsTabs.length;
  activeTab.value = settingsTabs[nextIndex];
}

watch(open, async (isOpen) => {
  if (!isOpen) return;
  await nextTick();
  focusSettingsNav();
});
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="overlay" @click="onOverlayClick">
      <div
        class="dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="settings-title"
      >
        <div class="dialog-header">
          <h2 id="settings-title">Settings</h2>
          <button class="close-btn" @click="closeSettings">&#x00D7;</button>
        </div>
        <div class="dialog-body">
          <nav
            ref="navEl"
            class="dialog-nav"
            tabindex="0"
            aria-label="Settings options"
            @keydown="onNavKeydown"
          >
            <p class="nav-section">Options</p>
            <button
              :class="{ active: activeTab === 'general' }"
              @click="activeTab = 'general'"
            >
              <span class="tab-icon">&#x25CE;</span>General
            </button>
            <button
              :class="{ active: activeTab === 'settings' }"
              @click="activeTab = 'settings'"
            >
              <span class="tab-icon">&#x2699;</span>Settings
            </button>
            <button
              :class="{ active: activeTab === 'theme' }"
              @click="activeTab = 'theme'"
            >
              <span class="tab-icon">&#x25D1;</span>Appearance
            </button>
            <button
              :class="{ active: activeTab === 'hotkeys' }"
              @click="activeTab = 'hotkeys'"
            >
              <span class="tab-icon">&#x2318;</span>Hotkeys
            </button>
          </nav>
          <div ref="contentEl" class="dialog-content">
            <section v-if="activeTab === 'general'" class="settings-section">
              <h3>General</h3>
              <div class="settings-card">
                <div class="setting-row">
                  <div class="setting-copy">
                    <p class="setting-title">Version</p>
                    <p class="setting-description">
                      Current vbdb-scout application version.
                    </p>
                  </div>
                  <p class="setting-value">0.1</p>
                </div>
                <div class="setting-row">
                  <div class="setting-copy">
                    <p class="setting-title">Stored scout files</p>
                    <p class="setting-description">
                      Imported .dvw copies edited by code changes.
                    </p>
                    <p class="path-value">{{ storedScoutFilesPath }}</p>
                  </div>
                  <button
                    class="secondary-btn"
                    @click="copyStoredScoutFilesPath"
                  >
                    Copy path
                  </button>
                </div>
              </div>
            </section>
            <section v-if="activeTab === 'settings'" class="settings-section">
              <h3>Settings</h3>
              <div class="settings-card">
                <div class="setting-row">
                  <div class="setting-copy">
                    <p class="setting-title">Autoplay on seek</p>
                    <p class="setting-description">
                      When navigating to the next play, automatically start
                      video playback. Turn off to keep the video paused when
                      seeking.
                    </p>
                  </div>
                  <label class="toggle">
                    <input
                      type="checkbox"
                      :checked="autoPlayOnSeek"
                      @change="
                        onAutoPlayOnSeekToggle(
                          ($event.target as HTMLInputElement).checked,
                        )
                      "
                    />
                    <span class="toggle-slider"></span>
                  </label>
                </div>
                <div class="setting-row">
                  <div class="setting-copy">
                    <p class="setting-title">Auto-advance montage clips</p>
                    <p class="setting-description">
                      When clip end time is reached, automatically move to the
                      next filtered row. Turn off to stop at clip end.
                    </p>
                  </div>
                  <label class="toggle">
                    <input
                      type="checkbox"
                      :checked="autoAdvanceMontage"
                      @change="
                        onAutoAdvanceMontageToggle(
                          ($event.target as HTMLInputElement).checked,
                        )
                      "
                    />
                    <span class="toggle-slider"></span>
                  </label>
                </div>
                <div class="setting-row">
                  <div class="setting-copy">
                    <p class="setting-title">
                      Auto select all plays when filtering
                    </p>
                    <p class="setting-description">
                      Automatically check every visible play in the code window
                      after filters are applied.
                    </p>
                  </div>
                  <label class="toggle">
                    <input
                      type="checkbox"
                      :checked="autoSelectFilteredPlays"
                      @change="
                        onAutoSelectFilteredPlaysToggle(
                          ($event.target as HTMLInputElement).checked,
                        )
                      "
                    />
                    <span class="toggle-slider"></span>
                  </label>
                </div>
                <div class="setting-row">
                  <div class="setting-copy">
                    <p class="setting-title">Code edit saving</p>
                    <p class="setting-description">
                      Save code edits immediately or keep them pending until you
                      choose to save.
                    </p>
                  </div>
                  <select
                    class="select-input"
                    :value="codeEditSaveMode"
                    @change="
                      onCodeEditSaveModeChange(
                        ($event.target as HTMLSelectElement).value,
                      )
                    "
                  >
                    <option value="after_edit">Save after editing code</option>
                    <option value="end_of_editing">
                      Save at end of editing
                    </option>
                  </select>
                </div>
                <div class="setting-row">
                  <div class="setting-copy">
                    <p class="setting-title">Columns shown</p>
                    <p class="setting-description">
                      Hide optional columns in the scout table. Code always
                      stays visible.
                    </p>
                  </div>
                  <details class="checkbox-dropdown">
                    <summary>Choose columns</summary>
                    <label>
                      <input
                        type="checkbox"
                        :checked="scoutVisibleColumns.videoTime"
                        @change="
                          onScoutColumnToggle(
                            'videoTime',
                            ($event.target as HTMLInputElement).checked,
                          )
                        "
                      />
                      Video time
                    </label>
                    <label>
                      <input
                        type="checkbox"
                        :checked="scoutVisibleColumns.set"
                        @change="
                          onScoutColumnToggle(
                            'set',
                            ($event.target as HTMLInputElement).checked,
                          )
                        "
                      />
                      Set
                    </label>
                    <label>
                      <input
                        type="checkbox"
                        :checked="scoutVisibleColumns.score"
                        @change="
                          onScoutColumnToggle(
                            'score',
                            ($event.target as HTMLInputElement).checked,
                          )
                        "
                      />
                      Score
                    </label>
                  </details>
                </div>
                <div class="setting-row">
                  <div class="setting-copy">
                    <p class="setting-title">Mute on start</p>
                    <p class="setting-description">Start video muted.</p>
                  </div>
                  <label class="toggle">
                    <input
                      type="checkbox"
                      :checked="muteOnAutoplayStart"
                      @change="
                        onMuteOnAutoplayStartToggle(
                          ($event.target as HTMLInputElement).checked,
                        )
                      "
                    />
                    <span class="toggle-slider"></span>
                  </label>
                </div>
                <div class="setting-row">
                  <div class="setting-copy">
                    <p class="setting-title">
                      Rewind / fast-forward step (seconds)
                    </p>
                    <p class="setting-description">
                      Used by the rewind and fast-forward hotkeys.
                    </p>
                  </div>
                  <input
                    class="number-input"
                    type="number"
                    min="0.1"
                    step="0.5"
                    :value="videoSeekStepSeconds"
                    @change="
                      onVideoSeekStepChange(
                        ($event.target as HTMLInputElement).value,
                      )
                    "
                  />
                </div>
                <div class="setting-row">
                  <div class="setting-copy">
                    <p class="setting-title">Uploading Scout Files</p>
                    <p class="setting-description">
                      Auto place scout files into seasons by year. When off,
                      files go to the selected season.
                    </p>
                  </div>
                  <label class="toggle">
                    <input
                      type="checkbox"
                      :checked="autoSeason"
                      @change="
                        onAutoSeasonToggle(
                          ($event.target as HTMLInputElement).checked,
                        )
                      "
                    />
                    <span class="toggle-slider"></span>
                  </label>
                </div>
              </div>
            </section>
            <section v-if="activeTab === 'theme'" class="settings-section">
              <h3>Appearance</h3>
              <div class="settings-card">
                <div class="setting-row">
                  <div class="setting-copy">
                    <p class="setting-title">Theme</p>
                    <p class="setting-description">
                      Choose the color palette used by the app.
                    </p>
                  </div>
                  <ThemeSelect />
                </div>
                <div class="setting-row">
                  <div class="setting-copy">
                    <p class="setting-title">Explorer scout list</p>
                    <p class="setting-description">
                      Choose a compact file list or the larger card list.
                    </p>
                  </div>
                  <select
                    class="select-input"
                    :value="explorerListStyle"
                    @change="
                      onExplorerListStyleChange(
                        ($event.target as HTMLSelectElement).value,
                      )
                    "
                  >
                    <option value="file">File list</option>
                    <option value="card">Card list</option>
                  </select>
                </div>
                <div class="setting-row">
                  <div class="setting-copy">
                    <p class="setting-title">Filenames</p>
                    <p class="setting-description">
                      Show prepending date before the teams playing in the
                      explorer.
                    </p>
                  </div>
                  <label class="toggle">
                    <input
                      type="checkbox"
                      :checked="showPrependingDate"
                      @change="
                        onShowPrependingDateToggle(
                          ($event.target as HTMLInputElement).checked,
                        )
                      "
                    />
                    <span class="toggle-slider"></span>
                  </label>
                </div>
              </div>
            </section>
            <section
              v-if="activeTab === 'hotkeys'"
              class="settings-section hotkeys-section"
            >
              <h3>Hotkeys</h3>
              <div class="settings-card hotkeys-card">
                <HotkeySettings />
              </div>
            </section>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  background:
    radial-gradient(
      circle at 50% 20%,
      color-mix(in srgb, var(--accent) 14%, transparent),
      transparent 28rem
    ),
    rgba(0, 0, 0, 0.58);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 200;
  backdrop-filter: blur(7px);
}

.dialog {
  position: relative;
  overflow: hidden;
  background: color-mix(in srgb, var(--bg) 92%, #111);
  border: 1px solid var(--border-soft);
  border-radius: 10px;
  width: min(1100px, calc(100vw - 72px));
  height: min(760px, calc(100vh - 72px));
  box-shadow: var(--shadow-lg);
}

.dialog-header {
  position: absolute;
  top: 8px;
  right: 10px;
  z-index: 1;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0;
}

.dialog-header h2 {
  position: absolute;
  width: 1px;
  height: 1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  clip-path: inset(50%);
  border: 0;
  padding: 0;
  margin: -1px;
  font-size: 15px;
  font-weight: 750;
  color: var(--fg);
  letter-spacing: -0.01em;
}

.close-btn {
  background: none;
  border: none;
  color: var(--fg);
  opacity: 0.62;
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  cursor: pointer;
  border-radius: 9px;
}

.close-btn:hover {
  opacity: 1;
  background: var(--surface-soft);
}

.dialog-body {
  display: flex;
  height: 100%;
}

.dialog-nav {
  width: 250px;
  padding: 34px 12px 18px 20px;
  border-right: 1px solid var(--border-soft);
  background: color-mix(in srgb, var(--bg) 86%, #101010);
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.nav-section {
  margin: 0 0 8px;
  color: var(--text-muted);
  font-size: 12px;
  font-weight: 650;
}

.dialog-nav button {
  position: relative;
  background: transparent;
  border: none;
  color: var(--fg);
  opacity: 0.9;
  padding: 6px 10px;
  text-align: left;
  font-size: 14px;
  font-family: inherit;
  cursor: pointer;
  border-radius: 5px;
  display: flex;
  align-items: center;
  gap: 9px;
  transition:
    opacity 140ms ease,
    background 140ms ease,
    color 140ms ease;
}

.tab-icon {
  width: 15px;
  color: var(--text-muted);
  font-size: 13px;
  text-align: center;
}

.dialog-nav button:hover {
  opacity: 1;
  background: color-mix(in srgb, var(--surface) 64%, transparent);
}

.dialog-nav button.active {
  opacity: 1;
  background: color-mix(in srgb, var(--surface) 76%, transparent);
  color: var(--fg);
}

.dialog-content {
  flex: 1;
  padding: 32px 60px 48px 48px;
  overflow: auto;
}

.settings-section {
  max-width: 740px;
}

.settings-section h3 {
  margin: 0 0 14px 16px;
  color: var(--fg);
  font-size: 15px;
  font-weight: 700;
}

.settings-card {
  overflow: hidden;
  border-radius: 10px;
  background: color-mix(in srgb, var(--surface) 54%, transparent);
}

.hotkeys-card {
  padding: 20px;
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 28px;
  padding: 18px 20px;
  border-bottom: 1px solid var(--border-soft);
}

.setting-row:last-child {
  border-bottom: 0;
}

.setting-copy {
  min-width: 0;
}

.setting-title {
  margin: 0 0 4px;
  color: var(--fg);
  font-size: 15px;
  line-height: 1.2;
}

.setting-description {
  margin: 0;
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.35;
}

.setting-value,
.version {
  color: var(--text-muted);
  font-size: 13px;
}

.setting-value {
  white-space: nowrap;
}

.path-value {
  margin: 6px 0 0;
  color: var(--text-muted);
  font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
  font-size: 11px;
  overflow-wrap: anywhere;
}

.secondary-btn {
  border: 1px solid var(--border-soft);
  background: var(--surface-soft);
  color: var(--fg);
  border-radius: 8px;
  padding: 6px 10px;
  font-size: 12px;
  cursor: pointer;
  white-space: nowrap;
}

.checkbox-dropdown {
  position: relative;
  min-width: 180px;
}

.checkbox-dropdown summary {
  border: 1px solid var(--border-soft);
  border-radius: 8px;
  background: color-mix(in srgb, var(--surface) 70%, transparent);
  color: var(--fg);
  font-size: 13px;
  padding: 6px 8px;
  cursor: pointer;
  list-style: none;
}

.checkbox-dropdown[open] summary {
  border-bottom-left-radius: 0;
  border-bottom-right-radius: 0;
}

.checkbox-dropdown label {
  display: flex;
  align-items: center;
  gap: 8px;
  background: var(--surface);
  border-left: 1px solid var(--border-soft);
  border-right: 1px solid var(--border-soft);
  padding: 8px;
  font-size: 12px;
}

.checkbox-dropdown label:last-child {
  border-bottom: 1px solid var(--border-soft);
  border-bottom-left-radius: 8px;
  border-bottom-right-radius: 8px;
}

.number-input,
.select-input {
  width: 88px;
  border: 1px solid var(--border-soft);
  border-radius: 8px;
  background: color-mix(in srgb, var(--surface) 70%, transparent);
  color: var(--fg);
  font-size: 13px;
  padding: 6px 8px;
}

.select-input {
  width: 180px;
}

.toggle {
  position: relative;
  display: inline-block;
  width: 44px;
  height: 24px;
  flex-shrink: 0;
}

.toggle input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  inset: 0;
  background: var(--surface-soft);
  border: 1px solid var(--border-soft);
  border-radius: 12px;
  cursor: pointer;
  transition:
    background 160ms ease,
    border-color 160ms ease;
}

.toggle-slider::before {
  content: "";
  position: absolute;
  left: 3px;
  top: 3px;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--fg);
  opacity: 0.6;
  transition:
    transform 160ms ease,
    opacity 160ms ease;
}

.toggle input:checked + .toggle-slider {
  background: var(--accent-soft);
  border-color: var(--accent-border);
}

.toggle input:checked + .toggle-slider::before {
  transform: translateX(20px);
  opacity: 1;
}
</style>
