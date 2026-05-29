<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import {
  getScoutRows,
  getScoutRowsMulti,
  getScoutRowsMultiFiltered,
  getScoutVideoPath,
} from "../../services/api/scoutFiles";
import {
  matchesHotkey,
  useExplorerHotkeys,
} from "../../composables/useExplorerHotkeys";
import type { ScoutPlayRow } from "../../types/database";

const props = defineProps<{
  matchIds: number[];
  clipStartOffset: number;
  clipEndOffset: number;
  codeFilters?: {
    relation?:
      | "equal"
      | "not_equal"
      | "next"
      | "previous"
      | "next_next"
      | "previous_previous"
      | "rally_contains";
    condition?: "contains" | "not_contains";
    team: string;
    number: string;
    skill: string;
    subType: string;
    grade: string;
    combo: string;
    startZone: string;
    endZone: string;
    skillType: string;
    players: string;
  }[];
}>();

const emit = defineEmits<{
  videoSource: [payload: { src: string; path: string; status: string }];
  seekTime: [payload: { time: number; clipEndTime: number | null }];
  togglePlayback: [];
  selectedClipsChange: [
    payload: {
      row_id: number;
      match_id: number | null;
      match_name: string | null;
      video_path: string;
      start_time: number;
      end_time: number;
      code: string;
      video_time_seconds: number;
    }[],
  ];
}>();

const rows = ref<ScoutPlayRow[]>([]);
const videoPaths = ref<Map<number, string>>(new Map());
const videoSourcePaths = ref<Map<number, string>>(new Map());
const loadingVideoMatchId = ref<number | null>(null);
const videoPathLoads = new Map<number, Promise<void>>();
const missingVideoMatchIds = new Set<number>();
const loading = ref(false);
const loadingStatus = ref("");
const error = ref("");
const activeRowId = ref<number | null>(null);
const activeRowMatchId = ref<number | null>(null);
const currentMatchId = ref<number | null>(null);
const tableScrollEl = ref<HTMLElement | null>(null);
const selectedRowIds = ref<Set<string>>(new Set());
const autoSelectFilteredPlays = ref(false);
const { hotkeys } = useExplorerHotkeys();
let seekRequestToken = 0;
let loadRequestToken = 0;
let loadDebounceTimer: ReturnType<typeof setTimeout> | null = null;
const LOAD_CHUNK_SIZE = 12;
const LARGE_SELECTION_LIMIT = 25;
const MAX_VISIBLE_ROWS = 5000;

function rowSelectionKey(row: ScoutPlayRow): string {
  return `${row.match_id ?? "single"}:${row.row_id}`;
}

function parseCodeFields(code: string | null | undefined) {
  const normalized = (code ?? "").toUpperCase().replace(/\s+/g, "");
  const main = normalized.split(";")[0] ?? "";
  const chars = [...main];
  const team = chars[0] ?? "";
  const number = /\d/.test(chars[1] ?? "") && /\d/.test(chars[2] ?? "")
    ? `${chars[1]}${chars[2]}`
    : "";
  const skill = number ? (chars[3] ?? "") : "";
  let grade = "";
  let combo = "";
  let subType = "";
  if (isGradeChar(chars[4] ?? "")) {
    grade = chars[4] ?? "";
    if (/[A-Z]/.test(chars[5] ?? "") && /\d/.test(chars[6] ?? "")) {
      combo = `${chars[5]}${chars[6]}`;
    }
  } else if (/[A-Z]/.test(chars[4] ?? "")) {
    subType = chars[4] ?? "";
    if (isGradeChar(chars[5] ?? "")) grade = chars[5] ?? "";
  }

  let startZone = "";
  let endZone = "";
  let skillType = "";
  const tailMatch = main.match(/~+([1-9])([1-9])([A-Z]?)([A-Z]?)(\d?)$/);
  if (tailMatch) {
    startZone = tailMatch[1] ?? "";
    endZone = tailMatch[2] ?? "";
    skillType = tailMatch[3] ?? "";
    subType = tailMatch[4] ?? "";
  }

  if (skill === "R") {
    const receptionLetters = [skillType, subType].filter((v) =>
      ["M", "R", "L", "W"].includes(v),
    );
    if (receptionLetters.length > 0) {
      skillType = receptionLetters[0] ?? "";
      subType = receptionLetters[1] ?? "";
    }
  }

  return {
    normalized,
    team,
    number,
    skill,
    subType,
    grade,
    combo,
    startZone,
    endZone,
    skillType,
    players: tailMatch?.[5] ?? "",
  };
}

function isGradeChar(value: string): boolean {
  return ["#", "!", "+", "-", "/", "="].includes(value);
}

function matchesField(value: string, query: string): boolean {
  const q = query.toUpperCase().replace(/\s+/g, "");
  if (!q) return true;
  return value.includes(q);
}

function matchesFilterRow(
  row: ScoutPlayRow,
  filter: {
    team: string;
    number: string;
    skill: string;
    subType: string;
    grade: string;
    combo: string;
    startZone: string;
    endZone: string;
    skillType: string;
    players: string;
  },
): boolean {
  const parsed = parseCodeFields(row.code);
  if (!matchesField(parsed.team, filter.team ?? "")) return false;
  if (!matchesField(parsed.number, filter.number ?? "")) return false;
  if (!matchesField(parsed.skill, filter.skill ?? "")) return false;
  if (!matchesField(parsed.subType, filter.subType ?? "")) return false;
  if (!matchesField(parsed.grade, filter.grade ?? "")) return false;
  if (!matchesField(parsed.combo, filter.combo ?? "")) return false;
  if (!matchesField(parsed.startZone, filter.startZone ?? "")) return false;
  if (!matchesField(parsed.endZone, filter.endZone ?? "")) return false;
  if (!matchesField(parsed.skillType, filter.skillType ?? "")) return false;
  if (!matchesField(parsed.players, filter.players ?? "")) return false;
  return true;
}

function rowMatchesCondition(
  row: ScoutPlayRow,
  filter: {
    condition?: "contains" | "not_contains";
    relation?: string;
    team: string;
    number: string;
    skill: string;
    subType: string;
    grade: string;
    combo: string;
    startZone: string;
    endZone: string;
    skillType: string;
    players: string;
  },
): boolean {
  const matched = matchesFilterRow(row, filter);
  return filter.condition === "not_contains" || filter.relation === "not_equal"
    ? !matched
    : matched;
}

function isActionRow(row: ScoutPlayRow): boolean {
  const parsed = parseCodeFields(row.code);
  return !!parsed.team && !!parsed.number && !!parsed.skill;
}

function filterHasFields(filter: NonNullable<typeof props.codeFilters>[number]): boolean {
  return [
    filter.team,
    filter.number,
    filter.skill,
    filter.subType,
    filter.grade,
    filter.combo,
    filter.startZone,
    filter.endZone,
    filter.skillType,
    filter.players,
  ].some((value) => (value ?? "").trim() !== "");
}

const hasActiveFilter = computed(() =>
  (props.codeFilters ?? []).some((filter) => filterHasFields(filter)),
);
const hasChainFilter = computed(() =>
  (props.codeFilters ?? []).slice(1).some((filter) => filterHasFields(filter)),
);
const canUseBackendBaseFilter = computed(() => {
  const baseFilter = props.codeFilters?.[0];
  if (!baseFilter || hasChainFilter.value) return false;
  if (baseFilter.condition === "not_contains") return false;
  if (baseFilter.relation === "not_equal") return false;
  return hasActiveFilter.value;
});

const filteredRows = computed(() => {
  const actionRows = rows.value.filter((row) => isActionRow(row));
  const chain = props.codeFilters ?? [];
  const baseFilter = chain[0];
  if (!baseFilter || !hasActiveFilter.value) return actionRows;

  const chainRows = chain.slice(1).filter((filter) => filterHasFields(filter));
  return actionRows.filter((row, rowIndex) => {
    if (!rowMatchesCondition(row, baseFilter)) return false;
    for (const filter of chainRows) {
      if (filter.relation === "rally_contains") {
        if (!rallyContains(actionRows, rowIndex, filter)) return false;
        continue;
      }
      const offset = relationOffset(filter.relation);
      const targetIndex = rowIndex + offset;
      const target = actionRows[targetIndex];
      if (!target) return false;
      if (!rowMatchesCondition(target, filter)) return false;
    }
    return true;
  });
});

const visibleRows = computed(() => filteredRows.value.slice(0, MAX_VISIBLE_ROWS));
const resultLimitStatus = computed(() => {
  if (filteredRows.value.length <= MAX_VISIBLE_ROWS) return "";
  return `Showing first ${MAX_VISIBLE_ROWS} of ${filteredRows.value.length} matching plays. Refine filters to narrow results.`;
});
const emptyRowsMessage = computed(() =>
  hasActiveFilter.value
    ? "No rows matched the current filter."
    : "No lines found after [3SCOUT].",
);

function relationOffset(relation: string | undefined): number {
  if (relation === "previous") return -1;
  if (relation === "previous_previous") return -2;
  if (relation === "next_next") return 2;
  return 1;
}

function rallyContains(
  actionRows: ScoutPlayRow[],
  rowIndex: number,
  filter: NonNullable<typeof props.codeFilters>[number],
): boolean {
  const matchId = actionRows[rowIndex]?.match_id ?? null;
  let start = rowIndex;
  while (start > 0) {
    const previous = actionRows[start - 1];
    if ((previous.match_id ?? null) !== matchId) break;
    if (parseCodeFields(previous.code).skill === "S") break;
    start -= 1;
  }

  let end = rowIndex;
  while (end + 1 < actionRows.length) {
    const next = actionRows[end + 1];
    if ((next.match_id ?? null) !== matchId) break;
    if (parseCodeFields(next.code).skill === "S") break;
    end += 1;
  }

  for (let index = start; index <= end; index += 1) {
    if (index !== rowIndex && rowMatchesCondition(actionRows[index], filter))
      return true;
  }
  return false;
}
const activeIndex = computed(() =>
  visibleRows.value.findIndex(
    (row) =>
      row.row_id === activeRowId.value &&
      (row.match_id ?? null) === activeRowMatchId.value,
  ),
);

onMounted(() => {
  loadAutoSelectSetting();
  window.addEventListener("storage", loadAutoSelectSetting);
  window.addEventListener("vbdb-settings-changed", loadAutoSelectSetting);
});

onBeforeUnmount(() => {
  if (loadDebounceTimer) clearTimeout(loadDebounceTimer);
  window.removeEventListener("storage", loadAutoSelectSetting);
  window.removeEventListener("vbdb-settings-changed", loadAutoSelectSetting);
});

watch(() => props.matchIds, scheduleLoadLines, { immediate: true, deep: true });
watch(() => props.codeFilters, scheduleLoadLines, { deep: true });
watch(
  visibleRows,
  () => {
    if (autoSelectFilteredPlays.value) {
      selectAllVisibleRows();
      return;
    }
    emitSelectedClips();
  },
  { immediate: true },
);
watch(autoSelectFilteredPlays, (enabled) => {
  if (enabled) selectAllVisibleRows();
});
watch(
  () => [props.clipStartOffset, props.clipEndOffset],
  () => emitSelectedClips(),
);
watch(
  rows,
  (nextRows) => {
    const validKeys = new Set(nextRows.map((row) => rowSelectionKey(row)));
    const nextSelected = new Set<string>();
    for (const key of selectedRowIds.value) {
      if (validKeys.has(key)) nextSelected.add(key);
    }
    if (nextSelected.size !== selectedRowIds.value.size) {
      selectedRowIds.value = nextSelected;
    }
  },
  { immediate: true },
);

function scheduleLoadLines() {
  if (loadDebounceTimer) clearTimeout(loadDebounceTimer);
  loadDebounceTimer = setTimeout(() => {
    loadDebounceTimer = null;
    void loadLines();
  }, 150);
}

function loadAutoSelectSetting() {
  autoSelectFilteredPlays.value =
    localStorage.getItem("autoSelectFilteredPlays") === "true";
}

async function loadLines() {
  const requestToken = ++loadRequestToken;
  loading.value = true;
  loadingStatus.value = "";
  error.value = "";
  rows.value = [];
  activeRowId.value = null;
  activeRowMatchId.value = null;
  currentMatchId.value = null;
  videoPaths.value = new Map();
  videoSourcePaths.value = new Map();
  videoPathLoads.clear();
  missingVideoMatchIds.clear();
  loadingVideoMatchId.value = null;
  selectedRowIds.value = new Set();

  if (props.matchIds.length === 0) {
    loading.value = false;
    loadingStatus.value = "";
    return;
  }

  try {
    if (props.matchIds.length === 1) {
      const mid = props.matchIds[0];
      loadingStatus.value = "Loading 1 file...";
      rows.value = await getScoutRows(mid);
      if (requestToken !== loadRequestToken) return;
      const linkedVideoPath = await getScoutVideoPath(mid);
      if (requestToken !== loadRequestToken) return;
      if (linkedVideoPath) {
        const src = toVideoSrc(linkedVideoPath);
        videoPaths.value.set(mid, src);
        videoSourcePaths.value.set(mid, linkedVideoPath);
        emit("videoSource", {
          src,
          path: linkedVideoPath,
          status: "Linked video loaded",
        });
        currentMatchId.value = mid;
      } else {
        emit("videoSource", {
          src: "",
          path: "",
          status: "No linked video found in this file",
        });
      }
      loadingStatus.value = "";
    } else {
      const total = props.matchIds.length;
      if (total > LARGE_SELECTION_LIMIT && !hasActiveFilter.value) {
        loading.value = false;
        loadingStatus.value = `${total} files selected. Add a filter before loading this many files, or select ${LARGE_SELECTION_LIMIT} or fewer files.`;
        const firstMatchId = props.matchIds[0];
        if (firstMatchId != null) {
          await ensureVideoLoaded(firstMatchId);
        }
        const firstSrc = firstMatchId != null ? videoPaths.value.get(firstMatchId) : "";
        if (firstMatchId != null && firstSrc) {
          currentMatchId.value = firstMatchId;
          emit("videoSource", {
            src: firstSrc,
            path: videoSourcePaths.value.get(firstMatchId) ?? "",
            status: `Loaded first selected video. Add a filter before loading ${total} files.`,
          });
        } else {
          emit("videoSource", {
            src: "",
            path: "",
            status: "Add a filter before loading all selected files",
          });
        }
        return;
      }
      let loaded = 0;
      const allRows: ScoutPlayRow[] = [];
      loadingStatus.value = `Loading files 0/${total}...`;
      if (canUseBackendBaseFilter.value) {
        loadingStatus.value = `Preparing cached scout rows for ${total} files...`;
        rows.value = await getScoutRowsMultiFiltered(props.matchIds, props.codeFilters ?? []);
        loaded = total;
        loadingStatus.value = `Loading files ${loaded}/${total}...`;
      } else {
        for (let i = 0; i < total; i += LOAD_CHUNK_SIZE) {
        const chunkIds = props.matchIds.slice(i, i + LOAD_CHUNK_SIZE);
        const chunkRows = await getScoutRowsMulti(chunkIds);
        if (requestToken !== loadRequestToken) return;
        allRows.push(...chunkRows);
        loaded += chunkIds.length;
        loadingStatus.value = `Loading files ${loaded}/${total}...`;
        }
        rows.value = allRows;
      }
      if (requestToken !== loadRequestToken) return;
      await nextTick();
      if (requestToken !== loadRequestToken) return;
      const firstRow = visibleRows.value[0];
      if (firstRow) {
        await seekToRow(firstRow);
      } else {
        emit("videoSource", {
          src: "",
          path: "",
          status: "No matching rows found",
        });
      }
      loadingStatus.value = "";
    }
  } catch (e) {
    if (requestToken !== loadRequestToken) return;
    error.value = e instanceof Error ? e.message : String(e);
    emit("videoSource", { src: "", path: "", status: "" });
  } finally {
    if (requestToken === loadRequestToken) {
      loading.value = false;
      if (!error.value) loadingStatus.value = "";
    }
  }
}

function toVideoSrc(path: string): string {
  const normalizedPath = path.replace(/\\/g, "/");
  if (
    normalizedPath.startsWith("http://") ||
    normalizedPath.startsWith("https://")
  )
    return normalizedPath;
  return convertFileSrc(normalizedPath);
}

async function ensureVideoLoaded(matchId: number) {
  if (videoPaths.value.has(matchId) || missingVideoMatchIds.has(matchId))
    return;
  const existing = videoPathLoads.get(matchId);
  if (existing) {
    await existing;
    return;
  }
  const load = (async () => {
    loadingVideoMatchId.value = matchId;
    const linkedVideoPath = await getScoutVideoPath(matchId);
    if (!linkedVideoPath) {
      missingVideoMatchIds.add(matchId);
      return;
    }
    videoPaths.value.set(matchId, toVideoSrc(linkedVideoPath));
    videoSourcePaths.value.set(matchId, linkedVideoPath);
  })();
  videoPathLoads.set(matchId, load);
  try {
    await load;
  } finally {
    videoPathLoads.delete(matchId);
    if (loadingVideoMatchId.value === matchId) loadingVideoMatchId.value = null;
  }
}

async function maybeSwitchVideo(row: ScoutPlayRow) {
  if (row.match_id == null) return;
  if (row.match_id === currentMatchId.value) return;
  const previousMatchId = currentMatchId.value;
  const previousSrc =
    previousMatchId != null
      ? (videoPaths.value.get(previousMatchId) ?? "")
      : "";
  const previousPath =
    previousMatchId != null
      ? (videoSourcePaths.value.get(previousMatchId) ?? "")
      : "";
  const targetMatchId = row.match_id;
  emit("videoSource", {
    src: previousSrc,
    path: previousPath,
    status: "Loading linked video...",
  });
  await ensureVideoLoaded(targetMatchId);
  const src = videoPaths.value.get(targetMatchId);
  if (src) {
    currentMatchId.value = targetMatchId;
    emit("videoSource", {
      src,
      path: videoSourcePaths.value.get(targetMatchId) ?? "",
      status:
        "Switched video for " + (row.match_name ?? `Match ${targetMatchId}`),
    });
    return;
  }
  emit("videoSource", {
    src: previousSrc,
    path: previousPath,
    status:
      "No linked video found for " +
      (row.match_name ?? `Match ${targetMatchId}`),
  });
}

function formatVideoTime(row: ScoutPlayRow): string {
  if (row.video_time_raw != null) return row.video_time_raw;
  if (row.video_time_seconds == null) return "-";
  return `${Math.floor(row.video_time_seconds)}`;
}

async function seekToRow(row: ScoutPlayRow) {
  const requestToken = ++seekRequestToken;
  activeRowId.value = row.row_id;
  activeRowMatchId.value = row.match_id ?? null;
  requestAnimationFrame(() => {
    const container = tableScrollEl.value;
    if (!container) return;
    const activeRow = container.querySelector<HTMLTableRowElement>(
      `tr[data-match-id="${row.match_id ?? ""}"][data-row-id="${row.row_id}"]`,
    );
    activeRow?.scrollIntoView({ block: "nearest" });
  });
  const applySeek = () => {
    if (requestToken !== seekRequestToken) return;
    if (activeRowId.value !== row.row_id) return;
    if (activeRowMatchId.value !== (row.match_id ?? null)) return;
    const time = row.video_time_seconds;
    if (time == null) return;
    const seekStart = Math.max(0, time + props.clipStartOffset);
    let seekEnd = time + props.clipEndOffset;
    if (seekEnd <= seekStart) seekEnd = seekStart + 0.25;
    emit("seekTime", { time: seekStart, clipEndTime: seekEnd });
  };

  if (row.match_id != null && row.match_id !== currentMatchId.value) {
    await maybeSwitchVideo(row);
    applySeek();
    return;
  }

  applySeek();
}

function moveSelection(next: boolean) {
  if (visibleRows.value.length === 0) return;
  const index = activeIndex.value;
  const targetIndex = next
    ? index < visibleRows.value.length - 1
      ? index + 1
      : 0
    : index > 0
      ? index - 1
      : visibleRows.value.length - 1;
  seekToRow(visibleRows.value[targetIndex]);
}

function onTableKeydown(e: KeyboardEvent) {
  if (visibleRows.value.length === 0) return;

  if (matchesHotkey(e, hotkeys.value.nextPlay)) {
    e.preventDefault();
    moveSelection(true);
    return;
  }

  if (matchesHotkey(e, hotkeys.value.previousPlay)) {
    e.preventDefault();
    moveSelection(false);
    return;
  }

  if (e.key === "Enter") {
    e.preventDefault();
    const row =
      visibleRows.value[activeIndex.value >= 0 ? activeIndex.value : 0];
    if (row) seekToRow(row);
    return;
  }

  if (matchesHotkey(e, hotkeys.value.togglePlaySelection)) {
    e.preventDefault();
    toggleActiveRowSelection();
    return;
  }

  if (e.key.toLowerCase() === "a") {
    e.preventDefault();
    selectAllVisibleRows();
    return;
  }

  if (e.key.toLowerCase() === "x") {
    e.preventDefault();
    clearAllVisibleRows();
  }
}

function isRowSelected(row: ScoutPlayRow): boolean {
  return selectedRowIds.value.has(rowSelectionKey(row));
}

function toggleRowSelection(row: ScoutPlayRow) {
  const next = new Set(selectedRowIds.value);
  const key = rowSelectionKey(row);
  if (next.has(key)) next.delete(key);
  else next.add(key);
  selectedRowIds.value = next;
  emitSelectedClips();
}

function toggleActiveRowSelection() {
  if (visibleRows.value.length === 0) return;
  const row = visibleRows.value[activeIndex.value >= 0 ? activeIndex.value : 0];
  if (!row) return;
  if (activeRowId.value == null) seekToRow(row);
  toggleRowSelection(row);
}

function selectAllVisibleRows() {
  const next = new Set(selectedRowIds.value);
  for (const row of visibleRows.value) next.add(rowSelectionKey(row));
  selectedRowIds.value = next;
  emitSelectedClips();
}

function clearAllVisibleRows() {
  selectedRowIds.value = new Set();
  emitSelectedClips();
}

function getRowVideoPath(row: ScoutPlayRow): string {
  if (row.match_id != null) {
    const path = videoSourcePaths.value.get(row.match_id);
    if (path) return path;
  }
  if (props.matchIds.length === 1) {
    const path = videoSourcePaths.value.get(props.matchIds[0]);
    if (path) return path;
  }
  return "";
}

function getSelectedMontageClips() {
  if (selectedRowIds.value.size === 0) return [];
  const clips: {
    row_id: number;
    match_id: number | null;
    match_name: string | null;
    video_path: string;
    start_time: number;
    end_time: number;
    code: string;
    video_time_seconds: number;
  }[] = [];

  for (const row of visibleRows.value) {
    if (!selectedRowIds.value.has(rowSelectionKey(row))) continue;
    if (row.video_time_seconds == null) continue;
    const videoPath = getRowVideoPath(row);
    if (!videoPath) continue;
    const start = Math.max(0, row.video_time_seconds + props.clipStartOffset);
    let end = row.video_time_seconds + props.clipEndOffset;
    if (end <= start) end = start + 0.25;
    clips.push({
      row_id: row.row_id,
      match_id: row.match_id ?? null,
      match_name: row.match_name ?? null,
      video_path: videoPath,
      start_time: start,
      end_time: end,
      code: row.code,
      video_time_seconds: row.video_time_seconds,
    });
  }

  return clips;
}

function emitSelectedClips() {
  emit("selectedClipsChange", getSelectedMontageClips());
}

function jumpByMatches(step: 1 | -1, count = 1) {
  if (visibleRows.value.length === 0) return;
  let index = activeIndex.value;
  if (index < 0) index = step > 0 ? -1 : 0;
  for (let i = 0; i < Math.max(1, count); i += 1) {
    index =
      (index + step + visibleRows.value.length) % visibleRows.value.length;
  }
  seekToRow(visibleRows.value[index]);
}

function playNextMontageRow(): boolean {
  if (visibleRows.value.length === 0) return false;
  const index = activeIndex.value;
  if (index < 0) {
    seekToRow(visibleRows.value[0]);
    return true;
  }
  if (index >= visibleRows.value.length - 1) return false;
  seekToRow(visibleRows.value[index + 1]);
  return true;
}

defineExpose({
  searchNext(count = 1) {
    jumpByMatches(1, count);
  },
  searchPrevious(count = 1) {
    jumpByMatches(-1, count);
  },
  playNextMontageRow,
  getSelectedMontageClips,
  toggleActiveSelection() {
    toggleActiveRowSelection();
  },
});
</script>

<template>
  <div class="scout-box" @keydown="onTableKeydown">
    <p v-if="loading && rows.length === 0" class="muted padded">
      {{ loadingStatus || "Loading..." }}
    </p>
    <p v-else-if="error" class="error padded">{{ error }}</p>
    <p v-else-if="loadingStatus" class="muted padded">
      {{ loadingStatus }}
    </p>
    <p v-else-if="rows.length === 0" class="muted padded">
      {{ emptyRowsMessage }}
    </p>

    <template v-else>
      <p v-if="loading" class="muted padded">
        {{ loadingStatus || "Loading..." }}
      </p>
      <p v-else-if="resultLimitStatus" class="muted padded">
        {{ resultLimitStatus }}
      </p>
      <div class="table-wrap">
        <div ref="tableScrollEl" class="table-scroll">
          <table class="scout-table">
            <colgroup>
              <col class="col-use" />
              <col v-if="matchIds.length > 1" class="col-match" />
              <col class="col-time" />
              <col class="col-set" />
              <col class="col-code" />
              <col class="col-score" />
            </colgroup>
            <thead>
              <tr>
                <th>
                  <button
                    class="use-clear-btn"
                    title="Clear all selected plays"
                    @click="clearAllVisibleRows"
                  >
                    Use
                  </button>
                </th>
                <th v-if="matchIds.length > 1">Match</th>
                <th>
                  Video time {{ filteredRows.length }}
                  /
                  {{ rows.length }}
                </th>
                <th>Set</th>
                <th>Code</th>
                <th>Score</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="row in visibleRows"
                :key="`${row.match_id}-${row.row_id}`"
                :data-match-id="row.match_id ?? ''"
                :data-row-id="row.row_id"
                :class="{
                  active:
                    activeRowId === row.row_id &&
                    (row.match_id ?? null) === activeRowMatchId,
                }"
                @click="seekToRow(row)"
              >
                <td class="use-cell">
                  <input
                    type="checkbox"
                    :checked="isRowSelected(row)"
                    @click.stop
                    @change="toggleRowSelection(row)"
                  />
                </td>
                <td v-if="matchIds.length > 1" class="match-label">
                  {{ row.match_name ?? "-" }}
                </td>
                <td>{{ formatVideoTime(row) }}</td>
                <td>{{ row.set_number ?? "-" }}</td>
                <td class="code">{{ row.code || "-" }}</td>
                <td>{{ row.score ?? "-" }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.scout-box {
  display: flex;
  flex-direction: column;
  gap: 4px;
  width: 100%;
  height: 100%;
  background: transparent;
  overflow: hidden;
}

.scout-box:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
}

.hint,
.muted {
  color: var(--text-muted);
  font-size: 12px;
  font-weight: 500;
}

.padded {
  padding: 0 10px;
}

.error {
  margin: 0;
  color: var(--red);
  font-size: 13px;
}

.table-wrap {
  flex: 1;
  border: 1px solid var(--border-soft);
  background: color-mix(in srgb, var(--bg) 86%, var(--surface));
  min-height: 120px;
  min-width: 160px;
  resize: both;
  overflow: hidden;
}

.table-scroll {
  width: 100%;
  height: 100%;
  overflow: auto;
}

.scout-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12px;
  table-layout: fixed;
}

.scout-table th,
.scout-table td {
  padding: 6px 8px;
  border-bottom: 1px solid var(--border-soft);
  text-align: left;
  overflow: hidden;
  text-overflow: ellipsis;
}

.scout-table th {
  position: sticky;
  top: 0;
  background: color-mix(in srgb, var(--surface) 80%, transparent);
  color: var(--text-muted);
  font-weight: 700;
  z-index: 1;
}

.scout-table tbody tr {
  cursor: pointer;
}

.scout-table tbody tr:hover {
  background: color-mix(in srgb, var(--surface) 78%, transparent);
}

.scout-table tbody tr.active {
  background: color-mix(in srgb, var(--accent-soft) 70%, transparent);
}

.code {
  font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
  white-space: nowrap;
}

.col-time {
  width: 88px;
}

.col-use {
  width: 46px;
}

.use-cell {
  text-align: center;
}

.use-clear-btn {
  border: 1px solid var(--border-soft);
  background: color-mix(in srgb, var(--surface) 72%, transparent);
  color: var(--text-muted);
  border-radius: 6px;
  padding: 2px 8px;
  font-size: 11px;
  cursor: pointer;
}

.use-clear-btn:hover {
  color: var(--fg);
  background: var(--surface-soft);
}

.col-set {
  width: 44px;
}

.col-score {
  width: 56px;
}

.col-match {
  width: 120px;
}

.match-label {
  color: var(--text-muted);
  font-size: 11px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
