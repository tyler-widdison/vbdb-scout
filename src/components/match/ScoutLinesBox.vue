<script setup lang="ts">
import {
  computed,
  nextTick,
  onBeforeUnmount,
  onMounted,
  ref,
  watch,
} from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import {
  getScoutRows,
  getScoutRowsMulti,
  getScoutRowsMultiFiltered,
  getScoutVideoPath,
  updateScoutCodes,
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
  editStatusChange: [payload: { status: string; dirtyCount: number }];
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
const codeEditSaveMode = ref<"after_edit" | "end_of_editing">("after_edit");
const editingKey = ref<string | null>(null);
const continuousEditMode = ref(false);
const lastEditField = ref("team");
const editingFields = ref({
  team: "",
  number: "",
  skill: "",
  subType: "",
  grade: "",
  startZone: "",
  endZone: "",
  skillType: "",
  players: "",
});
const originalEditingFields = ref({ ...editingFields.value });
const editStatus = ref("");
const editActiveIndex = ref<number | null>(null);
const visibleColumns = ref({ videoTime: true, set: true, score: true });
const dirtyCodes = ref<
  Map<string, { match_id: number; row_id: number; code: string }>
>(new Map());
let seekRequestToken = 0;
let loadRequestToken = 0;
let loadDebounceTimer: ReturnType<typeof setTimeout> | null = null;
let suppressNextBlurCommit = false;
const LOAD_CHUNK_SIZE = 12;
const LARGE_SELECTION_LIMIT = 25;
const MAX_VISIBLE_ROWS = 5000;

function rowSelectionKey(row: ScoutPlayRow): string {
  return `${row.match_id ?? "single"}:${row.row_id}`;
}

function rowEditKey(row: ScoutPlayRow): string {
  return `${row.match_id ?? props.matchIds[0] ?? "single"}:${row.row_id}`;
}

function rowMatchId(row: ScoutPlayRow): number | null {
  return (
    row.match_id ?? (props.matchIds.length === 1 ? props.matchIds[0] : null)
  );
}

function parseCodeFields(code: string | null | undefined) {
  const normalized = (code ?? "").toUpperCase().replace(/\s+/g, "");
  const main = normalized.split(";")[0] ?? "";
  const chars = [...main];
  const team = chars[0] ?? "";
  const number =
    /\d/.test(chars[1] ?? "") && /\d/.test(chars[2] ?? "")
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

  let skillType = "";
  let players = "";
  if ((chars[12] ?? "") !== "~") {
    skillType = chars[12] ?? "";
  }
  if ((chars[13] ?? "") !== "~") {
    players = chars[13] ?? "";
  }

  let startZone = "";
  let endZone = "";
  const tailParts = main.split("~");
  const tail = tailParts[tailParts.length - 1] ?? "";
  const tailChars = [...tail];
  if (
    tailParts.length > 1 &&
    tailChars.length >= 2 &&
    /\d/.test(tailChars[0]) &&
    /\d/.test(tailChars[1])
  ) {
    startZone = tailChars[0];
    endZone = tailChars[1];
    if (!skillType && /[A-Z]/.test(tailChars[2] ?? "")) {
      skillType = tailChars[2];
    }
    if (/[A-Z]/.test(tailChars[3] ?? "")) {
      subType = tailChars[3];
    }
    if (!players && /\d/.test(tailChars[4] ?? "")) {
      players = tailChars[4];
    }
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

  if (skill === "A" && players === "3") {
    players = "0";
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
    players,
  };
}

function codeToEditFields(code: string) {
  const parsed = parseCodeFields(code);
  return {
    team: parsed.team.toLowerCase(),
    number: parsed.number,
    skill: parsed.skill,
    subType: parsed.subType,
    grade: parsed.grade,
    startZone: parsed.startZone,
    endZone: parsed.endZone,
    skillType: parsed.skillType,
    players: parsed.players,
  };
}

function normalizeEditFields(fields: typeof editingFields.value) {
  return {
    team: fields.team.trim().toLowerCase().slice(0, 1),
    number: fields.number.replace(/\D/g, "").slice(0, 2),
    skill: fields.skill.trim().toUpperCase().slice(0, 1),
    subType: fields.subType.trim().toUpperCase().slice(0, 1),
    grade: fields.grade.trim().slice(0, 1),
    startZone: fields.startZone.replace(/\D/g, "").slice(0, 1),
    endZone: fields.endZone.replace(/\D/g, "").slice(0, 1),
    skillType: fields.skillType.trim().toUpperCase().slice(0, 1),
    players: fields.players.replace(/\D/g, "").slice(0, 1),
  };
}

function shouldShowAttackFields(fields: ReturnType<typeof codeToEditFields>) {
  return fields.skill.toUpperCase() === "A";
}

function applyCodeEditFields(code: string) {
  const fields = normalizeEditFields(editingFields.value);
  const chars = [...code];
  if (fields.team && chars.length > 0) chars[0] = fields.team;
  if (fields.number.length === 2 && chars.length > 2) {
    chars[1] = fields.number[0];
    chars[2] = fields.number[1];
  }
  if (fields.skill && chars.length > 3) chars[3] = fields.skill;

  const gradeIndex = chars.findIndex(
    (char, index) => index >= 4 && isGradeChar(char),
  );
  if (fields.grade && gradeIndex >= 0) chars[gradeIndex] = fields.grade;
  if (fields.subType && chars.length > 4) {
    const subIndex = gradeIndex === 4 ? 5 : 4;
    if (subIndex < chars.length && /[A-Za-z]/.test(chars[subIndex] ?? "")) {
      chars[subIndex] = fields.subType;
    }
  }

  const tailIndex = chars.lastIndexOf("~");
  if (tailIndex >= 0) {
    if (fields.startZone && tailIndex + 1 < chars.length)
      chars[tailIndex + 1] = fields.startZone;
    if (fields.endZone && tailIndex + 2 < chars.length)
      chars[tailIndex + 2] = fields.endZone;
    if (fields.skillType && tailIndex + 3 < chars.length)
      chars[tailIndex + 3] = fields.skillType;
    if (fields.players && tailIndex + 5 < chars.length)
      chars[tailIndex + 5] = fields.players;
  }
  if (fields.skillType && chars.length > 12) chars[12] = fields.skillType;
  if (fields.players && chars.length > 13) chars[13] = fields.players;
  return chars.join("");
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

function filterHasFields(
  filter: NonNullable<typeof props.codeFilters>[number],
): boolean {
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

const visibleRows = computed(() =>
  filteredRows.value.slice(0, MAX_VISIBLE_ROWS),
);
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
  loadSettings();
  window.addEventListener("storage", loadSettings);
  window.addEventListener("vbdb-settings-changed", loadSettings);
});

onBeforeUnmount(() => {
  if (loadDebounceTimer) clearTimeout(loadDebounceTimer);
  window.removeEventListener("storage", loadSettings);
  window.removeEventListener("vbdb-settings-changed", loadSettings);
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
watch([editStatus, dirtyCodes], () => emitEditStatus(), { deep: true });

function emitEditStatus() {
  emit("editStatusChange", {
    status: editStatus.value,
    dirtyCount: dirtyCodes.value.size,
  });
}

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

function loadCodeEditSetting() {
  codeEditSaveMode.value =
    localStorage.getItem("codeEditSaveMode") === "end_of_editing"
      ? "end_of_editing"
      : "after_edit";
}

function loadColumnSettings() {
  const stored = localStorage.getItem("scoutVisibleColumns");
  if (!stored) {
    visibleColumns.value = { videoTime: true, set: true, score: true };
    return;
  }
  try {
    const parsed = JSON.parse(stored) as Partial<typeof visibleColumns.value>;
    visibleColumns.value = {
      videoTime: parsed.videoTime !== false,
      set: parsed.set !== false,
      score: parsed.score !== false,
    };
  } catch {
    visibleColumns.value = { videoTime: true, set: true, score: true };
  }
}

function loadSettings() {
  loadAutoSelectSetting();
  loadCodeEditSetting();
  loadColumnSettings();
}

async function loadLines() {
  const requestToken = ++loadRequestToken;
  await promptSavePendingEdits();
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
  editingKey.value = null;

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
        const firstSrc =
          firstMatchId != null ? videoPaths.value.get(firstMatchId) : "";
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
        rows.value = await getScoutRowsMultiFiltered(
          props.matchIds,
          props.codeFilters ?? [],
        );
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

function startEditActive() {
  if (visibleRows.value.length === 0) return;
  const row = visibleRows.value[activeIndex.value >= 0 ? activeIndex.value : 0];
  if (!row) return;
  continuousEditMode.value = true;
  startEditRow(row);
}

function startEditRow(row: ScoutPlayRow) {
  activeRowId.value = row.row_id;
  activeRowMatchId.value = row.match_id ?? null;
  editActiveIndex.value = activeIndex.value >= 0 ? activeIndex.value : 0;
  editingKey.value = rowEditKey(row);
  editingFields.value = codeToEditFields(row.code);
  originalEditingFields.value = { ...editingFields.value };
  nextTick(() => {
    const input =
      tableScrollEl.value?.querySelector<HTMLInputElement>(
        `input[data-edit-key="${editingKey.value}"][data-edit-field="${lastEditField.value}"]`,
      ) ??
      tableScrollEl.value?.querySelector<HTMLInputElement>(
        `input[data-edit-key="${editingKey.value}"]`,
      );
    input?.focus();
    input?.select();
  });
}

function onEditFieldFocus(field: string) {
  lastEditField.value = field;
}

function cancelEdit(row?: ScoutPlayRow) {
  suppressNextBlurCommit = true;
  continuousEditMode.value = false;
  editingKey.value = null;
  if (row) {
    activeRowId.value = row.row_id;
    activeRowMatchId.value = row.match_id ?? null;
  }
  editActiveIndex.value = null;
}

async function restoreEditNavigation(row: ScoutPlayRow) {
  await nextTick();
  if (activeIndex.value >= 0) {
    editActiveIndex.value = null;
    return;
  }
  const fallbackIndex = Math.min(
    editActiveIndex.value ?? 0,
    Math.max(visibleRows.value.length - 1, 0),
  );
  const fallback = visibleRows.value[fallbackIndex];
  activeRowId.value = fallback?.row_id ?? row.row_id;
  activeRowMatchId.value = fallback
    ? (fallback.match_id ?? null)
    : (row.match_id ?? null);
  editActiveIndex.value = null;
}

async function commitEdit(row: ScoutPlayRow) {
  if (suppressNextBlurCommit) {
    suppressNextBlurCommit = false;
    return;
  }
  const matchId = rowMatchId(row);
  if (matchId == null) return;
  if (
    JSON.stringify(normalizeEditFields(editingFields.value)) ===
    JSON.stringify(normalizeEditFields(originalEditingFields.value))
  ) {
    editingKey.value = null;
    return;
  }
  const nextCode = applyCodeEditFields(row.code);
  editingKey.value = null;
  if (nextCode === row.code) return;

  row.code = nextCode;
  activeRowId.value = row.row_id;
  activeRowMatchId.value = row.match_id ?? null;
  emitSelectedClips();
  void restoreEditNavigation(row);
  if (codeEditSaveMode.value === "end_of_editing") {
    const next = new Map(dirtyCodes.value);
    next.set(rowEditKey(row), {
      match_id: matchId,
      row_id: row.row_id,
      code: nextCode,
    });
    dirtyCodes.value = next;
    editStatus.value = `${next.size} unsaved code edit${next.size === 1 ? "" : "s"}`;
    return;
  }

  editStatus.value = "Saving code...";
  try {
    await updateScoutCodes([
      { match_id: matchId, row_id: row.row_id, code: nextCode },
    ]);
    editStatus.value = "Code saved";
  } catch (e) {
    editStatus.value = e instanceof Error ? e.message : String(e);
  }
}

async function commitCurrentEdit() {
  if (!editingKey.value) return;
  const row = visibleRows.value.find(
    (item) => rowEditKey(item) === editingKey.value,
  );
  if (row) await commitEdit(row);
}

function onEditEscape(row: ScoutPlayRow) {
  cancelEdit(row);
  requestAnimationFrame(() => {
    const activeRow = tableScrollEl.value?.querySelector<HTMLTableRowElement>(
      `tr[data-match-id="${row.match_id ?? ""}"][data-row-id="${row.row_id}"]`,
    );
    activeRow?.scrollIntoView({ block: "nearest" });
  });
}

function onEditFocusout(row: ScoutPlayRow, event: FocusEvent) {
  const nextTarget = event.relatedTarget;
  if (
    nextTarget instanceof Node &&
    event.currentTarget instanceof HTMLElement
  ) {
    if (event.currentTarget.contains(nextTarget)) return;
  }
  void commitEdit(row);
}

async function savePendingEdits() {
  const changes = [...dirtyCodes.value.values()];
  if (changes.length === 0) return;
  editStatus.value = "Saving code edits...";
  await updateScoutCodes(changes);
  dirtyCodes.value = new Map();
  editStatus.value = "Code edits saved";
}

function discardPendingEdits() {
  dirtyCodes.value = new Map();
  editStatus.value = "Code edits discarded";
  void loadLines();
}

async function promptSavePendingEdits() {
  if (dirtyCodes.value.size === 0) return;
  const shouldSave = window.confirm("Save code edits to stored scout file?");
  if (shouldSave) await savePendingEdits();
  else dirtyCodes.value = new Map();
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

async function moveSelection(next: boolean) {
  if (visibleRows.value.length === 0) return;
  const shouldContinueEditing =
    continuousEditMode.value && editingKey.value != null;
  if (shouldContinueEditing) await commitCurrentEdit();
  const index = activeIndex.value;
  const targetIndex = next
    ? index < visibleRows.value.length - 1
      ? index + 1
      : 0
    : index > 0
      ? index - 1
      : visibleRows.value.length - 1;
  const target = visibleRows.value[targetIndex];
  await seekToRow(target);
  if (shouldContinueEditing) startEditRow(target);
}

async function onRowClick(row: ScoutPlayRow) {
  if (continuousEditMode.value && editingKey.value) await commitCurrentEdit();
  await seekToRow(row);
  if (continuousEditMode.value) startEditRow(row);
}

function onTableKeydown(e: KeyboardEvent) {
  if (visibleRows.value.length === 0) return;

  if (matchesHotkey(e, hotkeys.value.nextPlay)) {
    e.preventDefault();
    void moveSelection(true);
    return;
  }

  if (matchesHotkey(e, hotkeys.value.previousPlay)) {
    e.preventDefault();
    void moveSelection(false);
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

  if (e.ctrlKey && e.key.toLowerCase() === "e") {
    e.preventDefault();
    startEditActive();
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

function areAllVisibleRowsSelected(): boolean {
  return (
    visibleRows.value.length > 0 &&
    visibleRows.value.every((row) =>
      selectedRowIds.value.has(rowSelectionKey(row)),
    )
  );
}

function toggleAllVisibleRows() {
  if (areAllVisibleRowsSelected()) clearAllVisibleRows();
  else selectAllVisibleRows();
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
  startEditActive,
  savePendingEdits,
  discardPendingEdits,
  isEditingCode() {
    return continuousEditMode.value || editingKey.value != null;
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
      <div v-if="editStatus && dirtyCodes.size === 0" class="edit-status padded">
        <span>{{ editStatus }}</span>
      </div>
      <div class="table-wrap">
        <div ref="tableScrollEl" class="table-scroll">
          <table class="scout-table">
            <colgroup>
              <col class="col-use" />
              <col v-if="matchIds.length > 1" class="col-match" />
              <col v-if="visibleColumns.videoTime" class="col-time" />
              <col v-if="visibleColumns.set" class="col-set" />
              <col class="col-code" />
              <col v-if="visibleColumns.score" class="col-score" />
            </colgroup>
            <thead>
              <tr>
                <th>
                  <button
                    class="use-clear-btn"
                    title="Select or clear all visible plays"
                    @click="toggleAllVisibleRows"
                  >
                    Use
                  </button>
                </th>
                <th v-if="matchIds.length > 1">Match</th>
                <th v-if="visibleColumns.videoTime">
                  Video time {{ filteredRows.length }}
                  /
                  {{ rows.length }}
                </th>
                <th v-if="visibleColumns.set">Set</th>
                <th>Code</th>
                <th v-if="visibleColumns.score">Score</th>
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
                @click="onRowClick(row)"
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
                <td v-if="visibleColumns.videoTime">
                  {{ formatVideoTime(row) }}
                </td>
                <td v-if="visibleColumns.set">{{ row.set_number ?? "-" }}</td>
                <td class="code">
                  <div
                    v-if="editingKey === rowEditKey(row)"
                    class="code-edit-grid"
                    @click.stop
                    @keydown.enter.prevent.stop="commitEdit(row)"
                    @keydown.escape.prevent="onEditEscape(row)"
                    @focusout="onEditFocusout(row, $event)"
                  >
                    <input
                      v-model="editingFields.team"
                      class="code-edit-input tiny team"
                      :data-edit-key="rowEditKey(row)"
                      data-edit-field="team"
                      title="Team"
                      @focus="onEditFieldFocus('team')"
                    />
                    <input
                      v-model="editingFields.number"
                      class="code-edit-input number"
                      :data-edit-key="rowEditKey(row)"
                      data-edit-field="number"
                      title="Number"
                      @focus="onEditFieldFocus('number')"
                    />
                    <input
                      v-model="editingFields.skill"
                      class="code-edit-input tiny"
                      :data-edit-key="rowEditKey(row)"
                      data-edit-field="skill"
                      title="Skill"
                      @focus="onEditFieldFocus('skill')"
                    />
                    <input
                      v-model="editingFields.subType"
                      class="code-edit-input tiny"
                      :data-edit-key="rowEditKey(row)"
                      data-edit-field="subType"
                      title="Sub"
                      @focus="onEditFieldFocus('subType')"
                    />
                    <input
                      v-model="editingFields.grade"
                      class="code-edit-input tiny"
                      :data-edit-key="rowEditKey(row)"
                      data-edit-field="grade"
                      title="Grade"
                      @focus="onEditFieldFocus('grade')"
                    />
                    <input
                      v-model="editingFields.startZone"
                      class="code-edit-input tiny"
                      :data-edit-key="rowEditKey(row)"
                      data-edit-field="startZone"
                      title="Start"
                      @focus="onEditFieldFocus('startZone')"
                    />
                    <input
                      v-model="editingFields.endZone"
                      class="code-edit-input tiny"
                      :data-edit-key="rowEditKey(row)"
                      data-edit-field="endZone"
                      title="End"
                      @focus="onEditFieldFocus('endZone')"
                    />
                    <input
                      v-if="shouldShowAttackFields(editingFields)"
                      v-model="editingFields.skillType"
                      class="code-edit-input tiny"
                      :data-edit-key="rowEditKey(row)"
                      data-edit-field="skillType"
                      title="Type"
                      @focus="onEditFieldFocus('skillType')"
                    />
                    <input
                      v-if="shouldShowAttackFields(editingFields)"
                      v-model="editingFields.players"
                      class="code-edit-input tiny"
                      :data-edit-key="rowEditKey(row)"
                      data-edit-field="players"
                      title="Player"
                      @focus="onEditFieldFocus('players')"
                    />
                  </div>
                  <div v-else class="code-display-grid">
                    <span class="code-field team">{{
                      codeToEditFields(row.code).team || "-"
                    }}</span>
                    <span class="code-field number">{{
                      codeToEditFields(row.code).number || "--"
                    }}</span>
                    <span class="code-field">{{
                      codeToEditFields(row.code).skill || "-"
                    }}</span>
                    <span class="code-field">{{
                      codeToEditFields(row.code).subType || "-"
                    }}</span>
                    <span class="code-field">{{
                      codeToEditFields(row.code).grade || "-"
                    }}</span>
                    <span class="code-field">{{
                      codeToEditFields(row.code).startZone || "-"
                    }}</span>
                    <span class="code-field">{{
                      codeToEditFields(row.code).endZone || "-"
                    }}</span>
                    <span
                      v-if="shouldShowAttackFields(codeToEditFields(row.code))"
                      class="code-field"
                    >
                      {{ codeToEditFields(row.code).skillType || "-" }}
                    </span>
                    <span
                      v-if="shouldShowAttackFields(codeToEditFields(row.code))"
                      class="code-field"
                    >
                      {{ codeToEditFields(row.code).players || "-" }}
                    </span>
                  </div>
                </td>
                <td v-if="visibleColumns.score">{{ row.score ?? "-" }}</td>
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
  white-space: nowrap;
}

.code-edit-grid,
.code-display-grid {
  display: flex;
  gap: 3px;
  align-items: center;
}

.code-field {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 24px;
  height: 20px;
  border: 1px solid var(--border-soft);
  border-radius: 5px;
  background: color-mix(in srgb, var(--surface) 70%, transparent);
  color: var(--fg);
  font-size: 14px;
  line-height: 1;
}

.code-field.number {
  min-width: 34px;
}

.code-field.team {
  text-transform: lowercase;
}

.code-edit-input {
  border: 1px solid var(--accent-border);
  border-radius: 5px;
  background: var(--bg);
  color: var(--fg);
  font: inherit;
  padding: 2px 4px;
  text-transform: uppercase;
}

.code-edit-input.tiny {
  width: 24px;
}

.code-edit-input.number {
  width: 34px;
}

.code-edit-input.team {
  text-transform: lowercase;
}

.edit-status {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text-muted);
  font-size: 12px;
}

.edit-status-btn {
  border: 1px solid var(--border-soft);
  background: var(--surface-soft);
  color: var(--fg);
  border-radius: 6px;
  padding: 2px 8px;
  font-size: 11px;
  cursor: pointer;
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
