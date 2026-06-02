<script setup lang="ts">
import { ask, open } from "@tauri-apps/plugin-dialog";
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { useExplorerContext } from "../../composables/useExplorerContext";
import {
  matchesHotkey,
  useExplorerHotkeys,
} from "../../composables/useExplorerHotkeys";
import { getAutoSeason } from "../../services/api/settings";
import { importScoutFiles } from "../../services/api/scoutFiles";
import type { Match } from "../../types/database";
import * as api from "../../services/api";

const { hotkeys } = useExplorerHotkeys();

const props = withDefaults(
  defineProps<{
    allowMultiSelect?: boolean;
    maxVisibleFiles?: number;
    showHeaderActions?: boolean;
    showTeamFilter?: boolean;
    selectedMatchIds?: number[];
  }>(),
  {
    allowMultiSelect: true,
    maxVisibleFiles: 0,
    showHeaderActions: true,
    showTeamFilter: true,
    selectedMatchIds: () => [],
  },
);

type FileRow = Match & {
  seasonName: string;
  associationName: string;
  team_home?: string | null;
  team_away?: string | null;
  match_date?: string | null;
};

const emit = defineEmits<{
  openMatch: [payload: { id: number; name: string }];
  openMatches: [payload: { ids: number[] }];
}>();

const {
  associations,
  seasons,
  selectedSeasonIds,
  selectedAssociationId,
  selectedTeamNames,
  initExplorerContext,
  refreshAfterImport,
} = useExplorerContext();
const showVideoOnly = ref(false);
const allRows = ref<FileRow[]>([]);
const teams = ref<string[]>([]);
const teamSearch = ref("");

const rows = computed(() => {
  let filtered = allRows.value;
  if (teamSearch.value) {
    const q = teamSearch.value.toLowerCase();
    filtered = filtered.filter(
      (row) =>
        (row.team_home?.toLowerCase().includes(q) ?? false) ||
        (row.team_away?.toLowerCase().includes(q) ?? false),
    );
  }
  if (selectedTeamNames.value.length > 0) {
    const names = new Set(selectedTeamNames.value);
    filtered = filtered.filter(
      (row) => names.has(row.team_home ?? "") || names.has(row.team_away ?? ""),
    );
  }
  if (showVideoOnly.value) {
    filtered = filtered.filter((row) => row.has_video);
  }
  if (props.maxVisibleFiles > 0) {
    filtered = filtered.slice(0, props.maxVisibleFiles);
  }
  return filtered;
});
const activeMatchId = ref<number | null>(null);
const listEl = ref<HTMLElement | null>(null);
const uploading = ref(false);
const uploadError = ref("");
const uploadSuccess = ref("");
const selectedIds = ref<number[]>([]);
const syncingExternalSelection = ref(false);
const showPrependingDate = ref(false);

const selectedIndex = computed(() =>
  rows.value.findIndex((row) => row.id === activeMatchId.value),
);
const fileCountLabel = computed(
  () => `${rows.value.length} file${rows.value.length === 1 ? "" : "s"}`,
);
const selectedCount = computed(() => selectedIds.value.length);
const allVisibleSelected = computed(
  () =>
    rows.value.length > 0 &&
    rows.value.every((row) => selectedIds.value.includes(row.id)),
);

function toggleVideoOnly() {
  showVideoOnly.value = !showVideoOnly.value;
  if (
    activeMatchId.value &&
    !rows.value.some((row) => row.id === activeMatchId.value)
  ) {
    activeMatchId.value = rows.value[0]?.id ?? null;
  }
}

onMounted(async () => {
  await initExplorerContext();
  await refresh();
  loadShowPrependingDate();
  window.addEventListener("focus-season-explorer", focusExplorer);
  window.addEventListener("storage", loadShowPrependingDate);
  window.addEventListener("vbdb-settings-changed", loadShowPrependingDate);
});

onBeforeUnmount(() => {
  window.removeEventListener("focus-season-explorer", focusExplorer);
  window.removeEventListener("storage", loadShowPrependingDate);
  window.removeEventListener("vbdb-settings-changed", loadShowPrependingDate);
});

function loadShowPrependingDate() {
  showPrependingDate.value =
    localStorage.getItem("showPrependingDate") === "true";
}

function displayName(row: FileRow): string {
  const matchup =
    row.team_home && row.team_away
      ? `${row.team_home} vs ${row.team_away}`
      : row.name;
  if (showPrependingDate.value && row.match_date) {
    return `${row.match_date} ${matchup}`;
  }
  return matchup;
}

watch(
  selectedSeasonIds,
  async (ids) => {
    await refresh();
    if (ids.length === 0) {
      teams.value = [];
      return;
    }
    teams.value = await api.getTeamsForSeasons(ids);
  },
  { deep: true },
);

watch(
  selectedTeamNames,
  () => {
    if (
      activeMatchId.value &&
      !rows.value.some((row) => row.id === activeMatchId.value)
    ) {
      activeMatchId.value = rows.value[0]?.id ?? null;
    }
  },
  { deep: true },
);

watch(
  () => props.selectedMatchIds,
  (ids) => syncExternalSelection(ids),
  { deep: true, immediate: true },
);

watch(selectedIds, () => {
  if (syncingExternalSelection.value) return;
  emitSelectionFromCheckedRows();
});

function focusExplorer() {
  if (!activeMatchId.value && rows.value.length > 0)
    activeMatchId.value = rows.value[0].id;
  listEl.value?.focus();
}

let refreshPromise: Promise<void> | null = null;

async function refresh() {
  if (refreshPromise) return refreshPromise;
  refreshPromise = doRefresh();
  try {
    return await refreshPromise;
  } finally {
    refreshPromise = null;
  }
}

async function doRefresh() {
  const seasonMap = new Map(seasons.value.map((season) => [season.id, season]));
  const associationMap = new Map(
    associations.value.map((association) => [association.id, association]),
  );
  const chunks = await Promise.all(
    selectedSeasonIds.value.map((seasonId) => api.getMatches(seasonId)),
  );
  allRows.value = chunks.flatMap((chunk) =>
    chunk.map((match) => {
      const season = seasonMap.get(match.season_id);
      const association = season
        ? associationMap.get(season.association_id)
        : null;
      return {
        ...match,
        seasonName: season?.name ?? "Unknown season",
        associationName: association?.name ?? "Unknown association",
      };
    }),
  );

  if (
    activeMatchId.value &&
    !rows.value.some((row) => row.id === activeMatchId.value)
  ) {
    activeMatchId.value = rows.value[0]?.id ?? null;
  }

  const visible = new Set(rows.value.map((row) => row.id));
  selectedIds.value = selectedIds.value.filter((id) => visible.has(id));
  syncExternalSelection(props.selectedMatchIds);
}

function syncExternalSelection(ids: number[]) {
  if (!props.allowMultiSelect || ids.length === 0) return;
  const visible = new Set(rows.value.map((row) => row.id));
  const next = ids.filter((id) => visible.has(id));
  if (next.length === 0) return;
  if (sameIds(selectedIds.value, next)) return;
  syncingExternalSelection.value = true;
  selectedIds.value = next;
  activeMatchId.value = next[0];
  syncingExternalSelection.value = false;
  nextTick(scrollActiveIntoView);
}

function scrollActiveIntoView() {
  const el = listEl.value?.querySelector<HTMLDivElement>(".file-row.active");
  el?.scrollIntoView({ block: "nearest" });
}

function sameIds(left: number[], right: number[]) {
  return (
    left.length === right.length &&
    left.every((id, index) => id === right[index])
  );
}

function emitSelectionFromCheckedRows() {
  if (!props.allowMultiSelect) return;
  if (selectedIds.value.length > 1) {
    emit("openMatches", { ids: [...selectedIds.value] });
    return;
  }
  if (selectedIds.value.length === 1) {
    const id = selectedIds.value[0];
    const row = rows.value.find((value) => value.id === id);
    if (!row) return;
    activeMatchId.value = id;
    emit("openMatch", { id, name: row.name });
    return;
  }
  emit("openMatches", { ids: [] });
}

function openRow(id: number) {
  const row = rows.value.find((value) => value.id === id);
  if (!row) return;
  activeMatchId.value = id;

  if (props.allowMultiSelect) {
    toggleRowSelection(id);
    return;
  }

  emit("openMatch", { id: row.id, name: row.name });
}

async function renameSelected() {
  if (!activeMatchId.value) return;
  const row = rows.value.find((value) => value.id === activeMatchId.value);
  if (!row) return;
  const next = window.prompt("Rename scout file", row.name)?.trim();
  if (!next || next === row.name) return;
  await api.renameMatch(row.id, next);
  row.name = next;
}

async function deleteSelected() {
  if (!activeMatchId.value) return;
  await deleteOne(activeMatchId.value);
}

function toggleRowSelection(id: number) {
  if (!props.allowMultiSelect) return;
  if (selectedIds.value.includes(id)) {
    selectedIds.value = selectedIds.value.filter((value) => value !== id);
    return;
  }
  selectedIds.value = [...selectedIds.value, id];
}

function toggleSelectAll() {
  if (!props.allowMultiSelect) return;
  if (allVisibleSelected.value) {
    selectedIds.value = [];
    return;
  }
  selectedIds.value = rows.value.map((row) => row.id);
}

async function deleteSelectedRows() {
  if (!props.allowMultiSelect) return;
  if (selectedIds.value.length === 0) return;
  const confirmed = await ask(
    `Delete ${selectedIds.value.length} scout file(s)? This cannot be undone.`,
    {
      title: "Confirm bulk delete",
      kind: "warning",
      okLabel: "Delete all",
      cancelLabel: "Cancel",
    },
  );
  if (!confirmed) return;

  const ids = [...selectedIds.value];
  for (const id of ids) {
    await api.deleteMatch(id);
  }

  selectedIds.value = [];
  await refresh();
}

async function deleteOne(id: number) {
  const row = rows.value.find((value) => value.id === id);
  if (!row) return;

  const confirmed = await ask(`Delete "${row.name}"? This cannot be undone.`, {
    title: "Confirm delete",
    kind: "warning",
    okLabel: "Delete",
    cancelLabel: "Cancel",
  });
  if (!confirmed) return;

  const current = rows.value.findIndex((value) => value.id === id);
  await api.deleteMatch(id);
  await refresh();
  activeMatchId.value =
    rows.value[current]?.id ?? rows.value[current - 1]?.id ?? null;
}

async function onKeydown(e: KeyboardEvent) {
  if ((e.target as HTMLElement).tagName === "INPUT") return;
  e.stopPropagation();
  if (
    props.allowMultiSelect &&
    (e.key === " " || e.key === "Spacebar") &&
    activeMatchId.value
  ) {
    e.preventDefault();
    toggleRowSelection(activeMatchId.value);
    return;
  }

  if (e.key === "ArrowDown") {
    e.preventDefault();
    const next =
      selectedIndex.value < rows.value.length - 1 ? selectedIndex.value + 1 : 0;
    activeMatchId.value = rows.value[next].id;
    return;
  }

  if (e.key === "ArrowUp") {
    e.preventDefault();
    const prev =
      selectedIndex.value > 0 ? selectedIndex.value - 1 : rows.value.length - 1;
    activeMatchId.value = rows.value[prev].id;
    return;
  }

  if (e.key === "Enter" && activeMatchId.value) {
    e.preventDefault();
    if (
      props.allowMultiSelect &&
      selectedIds.value.length > 1 &&
      selectedIds.value.includes(activeMatchId.value)
    ) {
      emit("openMatches", { ids: [...selectedIds.value] });
    } else {
      openRow(activeMatchId.value);
    }
    return;
  }

  if (e.key === "F2") {
    e.preventDefault();
    await renameSelected();
    return;
  }

  if (matchesHotkey(e, hotkeys.value.deleteFile)) {
    e.preventDefault();
    await deleteSelected();
    return;
  }

  if (e.key === "Delete" || e.key === "Backspace") {
    e.preventDefault();
    await deleteSelected();
    return;
  }
}

async function uploadMatches() {
  uploadError.value = "";
  uploadSuccess.value = "";
  const selected = await open({
    multiple: true,
    filters: [{ name: "DataVolley", extensions: ["dvw"] }],
  });
  if (!selected) return;

  const paths = Array.isArray(selected) ? selected : [selected];
  if (paths.length === 0) return;

  const selectedAssociation = associations.value.find(
    (association) => association.id === selectedAssociationId.value,
  );
  const targetSeasons = seasons.value.filter((season) =>
    selectedSeasonIds.value.includes(season.id),
  );
  const fallbackSeason = targetSeasons[0]?.name ?? "Untitled season";
  const autoSeason = await getAutoSeason();

  uploading.value = true;
  try {
    const result = await importScoutFiles(
      paths,
      selectedAssociation?.name ?? "VBDB",
      fallbackSeason,
      autoSeason,
    );
    if (result.imported.length === 0 && result.failed.length > 0) {
      uploadError.value = `Import failed: ${result.failed[0].reason}`;
      return;
    }
    const parts: string[] = [];
    if (result.imported.length > 0) {
      parts.push(`Imported ${result.imported.length} file(s)`);
    }
    if (result.failed.length > 0) {
      parts.push(
        `${result.failed.length} file(s) skipped (${result.failed[0].reason})`,
      );
    }
    uploadSuccess.value = parts.join(". ");
    refreshAfterImport();
    window.dispatchEvent(new CustomEvent("scout-files-imported"));
  } catch (e) {
    uploadError.value = e instanceof Error ? e.message : String(e);
  } finally {
    uploading.value = false;
  }
}

defineExpose({ refresh });
</script>

<template>
  <div ref="listEl" class="season-tree" tabindex="0" @keydown="onKeydown">
    <input
      class="team-search"
      type="text"
      placeholder="Search teams..."
      v-model="teamSearch"
    />
    <div class="tree-header">
      <div class="tree-header-left">
        <span class="tree-hint">{{ fileCountLabel }}</span>
      </div>
      <div v-if="props.showHeaderActions" class="header-actions">
        <button
          v-if="props.allowMultiSelect"
          class="action-btn visible-files"
          :class="{ active: allVisibleSelected }"
          :title="
            allVisibleSelected ? 'Clear selection' : 'Select all visible files'
          "
          @click="toggleSelectAll"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            height="20px"
            viewBox="0 -960 960 960"
            width="20px"
            fill="#e3e3e3"
          >
            <path d="M400-301 240-461l51-51 109 109 269-269 51 51-320 320Z" />
          </svg>
        </button>
        <button
          v-if="props.allowMultiSelect"
          class="action-btn delete-selected"
          title="Delete selected files"
          :disabled="selectedCount === 0"
          @click="deleteSelectedRows"
          aria-label="Delete selected files"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            height="20px"
            viewBox="0 -960 960 960"
            width="20px"
            fill="#e3e3e3"
          >
            <path
              d="M312-144q-29.7 0-50.85-21.15Q240-186.3 240-216v-480h-48v-72h192v-48h192v48h192v72h-48v479.57Q720-186 698.85-165T648-144H312Zm336-552H312v480h336v-480ZM384-288h72v-336h-72v336Zm120 0h72v-336h-72v336ZM312-696v480-480Z"
            />
          </svg>
        </button>
        <button
          class="action-btn"
          title="Upload scout files"
          :disabled="uploading"
          @click="uploadMatches"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            height="20px"
            viewBox="0 -960 960 960"
            width="20px"
            fill="#e3e3e3"
          >
            <path
              d="M444-336v-342L339-573l-51-51 192-192 192 192-51 51-105-105v342h-72ZM263.72-192Q234-192 213-213.15T192-264v-72h72v72h432v-72h72v72q0 29.7-21.16 50.85Q725.68-192 695.96-192H263.72Z"
            />
          </svg>
        </button>
        <button
          class="action-btn"
          :class="{ active: showVideoOnly }"
          :title="
            showVideoOnly ? 'Show all files' : 'Show only files with video'
          "
          @click="toggleVideoOnly"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            height="20px"
            viewBox="0 -960 960 960"
            width="20px"
            fill="#e3e3e3"
          >
            <path
              d="M360-264h168q10.2 0 17.1-6.9 6.9-6.9 6.9-17.1v-50l72 50v-168l-72 50v-50q0-10.2-6.9-17.1-6.9-6.9-17.1-6.9H360q-10.2 0-17.1 6.9-6.9 6.9-6.9 17.1v168q0 10.2 6.9 17.1 6.9 6.9 17.1 6.9ZM263.72-96Q234-96 213-117.15T192-168v-624q0-29.7 21.15-50.85Q234.3-864 264-864h312l192 192v504q0 29.7-21.16 50.85Q725.68-96 695.96-96H263.72ZM528-624v-168H264v624h432v-456H528ZM264-792v189-189 624-624Z"
            />
          </svg>
        </button>
      </div>
    </div>

    <div class="tree-body">
      <div
        v-for="row in rows"
        :key="row.id"
        class="file-row"
        :class="{
          active: activeMatchId === row.id,
          selected: selectedIds.includes(row.id),
        }"
        @click="openRow(row.id)"
      >
        <label v-if="props.allowMultiSelect" class="row-select" @click.stop>
          <input
            class="checkbox-margin"
            type="checkbox"
            :checked="selectedIds.includes(row.id)"
            @change="toggleRowSelection(row.id)"
          />
        </label>
        <span class="name" :class="{ 'no-video': !row.has_video }">
          {{ displayName(row) }}
        </span>
        <span class="meta">
          {{ row.associationName }} / {{ row.seasonName }}
        </span>
        <span v-if="!showPrependingDate && row.match_date" class="match-date">{{
          row.match_date
        }}</span>
        <button
          class="remove"
          title="Delete file"
          @click.stop="deleteOne(row.id)"
        >
          ×
        </button>
      </div>
      <div v-if="rows.length === 0" class="empty-wrap">
        <p class="empty">No scout files for selected seasons</p>
        <button
          class="upload-empty-btn"
          :disabled="uploading"
          @click="uploadMatches"
        >
          {{ uploading ? "Importing..." : "Upload scout matches" }}
        </button>
      </div>
      <p v-if="uploadError" class="upload-error">{{ uploadError }}</p>
      <p v-if="uploadSuccess" class="upload-success">{{ uploadSuccess }}</p>
    </div>
  </div>
</template>

<style scoped>
.season-tree {
  display: flex;
  flex-direction: column;
  height: 100%;
  outline: none;
}

.tree-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-bottom: 1px solid var(--border-soft);
}

.tree-header-left {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
  min-width: 0;
}
.team-search {
  min-width: 0;
  border: 1px solid var(--border-soft);
  border-radius: 5px;
  background: var(--surface-soft);
  color: var(--fg);
  font-size: 12px;
  outline: none;
  padding: 4px 4px;
  margin-left: 4px;
  margin-right: 4px;
  margin-top: 2px;
  margin-bottom: 2px;
}

.team-search::placeholder {
  color: var(--text-muted);
}

.team-search:focus {
  border-color: var(--accent-border);
}

.team-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  max-height: 96px;
  overflow-y: auto;
}

.team-chip {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  font-size: 11px;
  color: var(--text-muted);
  cursor: pointer;
  padding: 2px 6px;
  border: 1px solid var(--border-soft);
  border-radius: 4px;
  background: transparent;
  user-select: none;
}

.team-chip input {
  display: none;
}

.team-chip:hover {
  border-color: var(--accent-border);
  color: var(--fg);
}

.team-chip.active {
  background: var(--accent-soft);
  border-color: var(--accent-border);
  color: var(--accent);
}

.team-empty {
  font-size: 11px;
  color: var(--text-muted);
}

.header-actions {
  display: flex;
  gap: 4px;
}

.tree-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--fg);
  opacity: 0.6;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.tree-hint {
  flex-shrink: 0;
  color: var(--text-muted);
  font-size: 11px;
  white-space: nowrap;
}

.action-btn {
  background: none;
  border: 1px solid transparent;
  color: var(--fg);
  opacity: 0.72;
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  cursor: pointer;
  border-radius: 7px;
}

.action-btn:hover {
  opacity: 1;
  background: var(--surface-soft);
  border-color: var(--border-soft);
}

.action-btn:disabled {
  opacity: 0.35;
  cursor: default;
}

.action-btn.active {
  opacity: 1;
  background: var(--accent-soft);
  border-color: var(--accent-border);
  color: var(--accent);
}

.delete-selected {
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.02em;
}

.tree-body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}

.file-row {
  position: relative;
  display: flex;
  flex-direction: column;
  border: 1px solid transparent;
  border-radius: 6px;
  background: none;
  color: var(--fg);
  text-align: left;
  cursor: pointer;
  padding: 6px 8px;
}

.file-row:hover {
  background: var(--surface-soft);
}

.file-row.active {
  background: var(--accent-soft);
  border-color: var(--accent-border);
}

.file-row.selected:not(.active) {
  border-color: var(--border-soft);
  background: color-mix(in srgb, var(--surface-soft) 80%, transparent);
}

.row-select {
  position: absolute;
  left: 8px;
  top: 50%;
  transform: translateY(-50%);
  display: inline-flex;
  align-items: center;
}

.row-select input {
  width: 14px;
  height: 14px;
}

.name {
  font-size: 13px;
  padding-left: 20px;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.name.no-video {
  background: color-mix(in srgb, var(--accent) 8%, transparent);
  border-radius: 3px;
}

.meta {
  font-size: 11px;
  color: var(--text-muted);
  padding-left: 20px;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.match-date {
  font-size: 11px;
  color: var(--text-muted);
  padding-left: 20px;
  flex-shrink: 0;
}

.remove {
  position: absolute;
  right: 8px;
  width: 20px;
  height: 20px;
  border: 1px solid transparent;
  border-radius: 5px;
  background: transparent;
  color: var(--text-muted);
  font-size: 14px;
  line-height: 1;
  cursor: pointer;
  opacity: 0;
}

.file-row:hover .remove,
.file-row.active .remove {
  opacity: 1;
}

.remove:hover {
  color: #e81123;
  border-color: color-mix(in srgb, #e81123 35%, var(--border-soft));
  background: color-mix(in srgb, #e81123 8%, transparent);
}

.empty-wrap {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 12px 8px;
}

.empty {
  margin: 0;
  color: var(--text-muted);
  font-size: 12px;
}

.upload-empty-btn {
  border: 1px solid var(--border-soft);
  border-radius: 8px;
  background: var(--surface-soft);
  color: var(--fg);
  cursor: pointer;
  font-size: 12px;
  padding: 6px 12px;
}

.upload-empty-btn:hover:not(:disabled) {
  background: var(--accent-soft);
  border-color: var(--accent-border);
  color: var(--accent);
}

.upload-empty-btn:disabled {
  opacity: 0.5;
  cursor: default;
}

.upload-error {
  margin: 6px 8px 0;
  color: var(--red);
  font-size: 12px;
}

.upload-success {
  margin: 6px 8px 0;
  color: #4caf50;
  font-size: 12px;
}

.visible-files {
  margin-left: 2px;
}
</style>
