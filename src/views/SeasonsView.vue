<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { useRoute } from "vue-router";
import { open, save } from "@tauri-apps/plugin-dialog";
import { convertFileSrc } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { exportMontageVideo } from "../services/api/scoutFiles";
import {
  matchesHotkey,
  useExplorerHotkeys,
} from "../composables/useExplorerHotkeys";
import {
  GRADES,
  SKILLS,
  ZONES,
  getSkillTypes,
  getSubTypes,
} from "../constants/datavolley";
import ExplorerDrawer from "../components/ExplorerDrawer.vue";
import MatchEmptyState from "../components/match/MatchEmptyState.vue";
import ScoutLinesBox from "../components/match/ScoutLinesBox.vue";
import MatchVideoPanel from "../components/match/MatchVideoPanel.vue";

const route = useRoute();
const explorerDrawer = ref<InstanceType<typeof ExplorerDrawer> | null>(null);
const scoutBox = ref<InstanceType<typeof ScoutLinesBox> | null>(null);
const videoPanel = ref<InstanceType<typeof MatchVideoPanel> | null>(null);
const { hotkeys } = useExplorerHotkeys();
const selectedMatchIds = ref<number[]>(
  route.query.matchId ? [Number(route.query.matchId)] : [],
);
const explorerVisible = ref(true);
const scoutWidth = ref(420);
const videoWidth = ref(560);
const linkedVideoSrc = ref("");
const manualVideoSrc = ref("");
const seekTime = ref<number | null>(null);
const clipEndTime = ref<number | null>(null);
const playToggleToken = ref(0);
const videoStatus = ref("");
const clipStartOffset = ref("0");
const clipEndOffset = ref("+8");
const clipStartInput = ref<HTMLInputElement | null>(null);
const clipEndInput = ref<HTMLInputElement | null>(null);
const advancedFiltersOpen = ref(false);
type RuleRelation =
  | "equal"
  | "not_equal"
  | "next"
  | "previous"
  | "next_next"
  | "previous_previous"
  | "rally_contains";
type ChainFilterRow = {
  relation: RuleRelation;
  condition: "contains" | "not_contains";
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
};

const ruleOptions: { value: RuleRelation; label: string }[] = [
  { value: "equal", label: "is equal" },
  { value: "not_equal", label: "is NOT equal" },
  { value: "previous", label: "was previous" },
  { value: "next", label: "was successive" },
  { value: "previous_previous", label: "was previous previous" },
  { value: "next_next", label: "was next next" },
  { value: "rally_contains", label: "rally contains" },
];

function createFilterRow(relation: RuleRelation = "equal"): ChainFilterRow {
  return {
    relation,
    condition: "contains",
    team: "",
    number: "",
    skill: "",
    subType: "",
    grade: "",
    combo: "",
    startZone: "",
    endZone: "",
    skillType: "",
    players: "",
  };
}

const codeFilters = ref<ChainFilterRow[]>([createFilterRow()]);
const scoutPos = ref({ x: 28, y: 150 });
const videoPos = ref({ x: 470, y: 150 });
const scoutHeight = ref(760);
const videoHeight = ref(760);
const panelOrder = ref<{ scout: number; video: number; filter: number }>({
  scout: 3,
  video: 2,
  filter: 1,
});
const zCounter = ref(3);

function bringPanelToFront(target: "scout" | "video" | "filter") {
  zCounter.value += 1;
  panelOrder.value = {
    ...panelOrder.value,
    [target]: zCounter.value,
  };
}

const filterPos = ref({ x: 12, y: 8 });
const autoPlayOnSeek = ref(true);
const autoAdvanceMontage = ref(true);
const muteOnAutoplayStart = ref(false);
const selectedMontageClips = ref<
  {
    row_id: number;
    match_id: number | null;
    match_name: string | null;
    video_path: string;
    start_time: number;
    end_time: number;
    code: string;
    video_time_seconds: number;
  }[]
>([]);
const exportStatus = ref("");
const exportRunning = ref(false);
const scoutEditStatus = ref("");
const scoutDirtyEditCount = ref(0);
const seekStepSeconds = ref(3);
let unlistenMontageProgress: UnlistenFn | null = null;

let dragState: {
  target:
    | "scout"
    | "video"
    | "filter"
    | "scout-resize"
    | "video-resize"
    | "scout-resize-right"
    | "video-resize-right"
    | "scout-resize-left"
    | "video-resize-left"
    | "scout-resize-top"
    | "video-resize-top"
    | "scout-resize-top-left"
    | "video-resize-top-left"
    | "scout-resize-top-right"
    | "video-resize-top-right"
    | "scout-resize-bottom-left"
    | "video-resize-bottom-left"
    | "scout-resize-bottom"
    | "video-resize-bottom";
  pointerId: number;
  startX: number;
  startY: number;
  originX: number;
  originY: number;
  originWidth?: number;
  originHeight?: number;
} | null = null;

const EDGE_RESIZE_ZONE = 10;
const EXPLORER_WIDTH = 260;
const COLLAPSED_EXPLORER_WIDTH = 52;

function getMinPanelX() {
  return explorerVisible.value ? -EXPLORER_WIDTH : -COLLAPSED_EXPLORER_WIDTH;
}

function onKeydown(e: KeyboardEvent) {
  if (scoutBox.value?.isEditingCode()) {
    if (matchesHotkey(e, hotkeys.value.pauseWhileEditingCode)) {
      e.preventDefault();
      onTogglePlayback();
      return;
    }
    if (matchesHotkey(e, hotkeys.value.rewindWhileEditingCode)) {
      e.preventDefault();
      videoPanel.value?.seekBy(-seekStepSeconds.value);
      return;
    }
  }

  if (e.shiftKey && e.key === "<") {
    e.preventDefault();
    focusClipOffset("start");
    return;
  }
  if (e.ctrlKey && (e.key === "." || e.key === ">")) {
    e.preventDefault();
    focusClipOffset("end");
    return;
  }

  if (isEditableTarget(e.target)) return;

  if (matchesHotkey(e, hotkeys.value.videoPlayPause)) {
    e.preventDefault();
    onTogglePlayback();
    return;
  }
  if (matchesHotkey(e, hotkeys.value.videoSeekForward)) {
    e.preventDefault();
    videoPanel.value?.seekBy(seekStepSeconds.value);
    return;
  }
  if (matchesHotkey(e, hotkeys.value.videoSeekBackward)) {
    e.preventDefault();
    videoPanel.value?.seekBy(-seekStepSeconds.value);
    return;
  }
  if (matchesHotkey(e, hotkeys.value.videoSeekForwardOneMinute)) {
    e.preventDefault();
    videoPanel.value?.seekBy(60);
    return;
  }
  if (matchesHotkey(e, hotkeys.value.nextPlay)) {
    e.preventDefault();
    scoutBox.value?.searchNext(1);
    return;
  }
  if (matchesHotkey(e, hotkeys.value.previousPlay)) {
    e.preventDefault();
    scoutBox.value?.searchPrevious(1);
    return;
  }
  if (matchesHotkey(e, hotkeys.value.togglePlaySelection)) {
    e.preventDefault();
    scoutBox.value?.toggleActiveSelection();
    return;
  }
  if (e.ctrlKey && e.key.toLowerCase() === "e") {
    e.preventDefault();
    scoutBox.value?.startEditActive();
    return;
  }

  if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === "c") {
    e.preventDefault();
    const filterInput = document.querySelector<HTMLInputElement>(
      ".code-filter-list input",
    );
    filterInput?.focus();
    filterInput?.select();
    return;
  }
  if (e.ctrlKey && e.key.toLowerCase() === "f") {
    e.preventDefault();
    const filterInput = document.querySelector<HTMLInputElement>(
      ".code-filter-list input",
    );
    filterInput?.focus();
    filterInput?.select();
    return;
  }
  if (matchesHotkey(e, hotkeys.value.toggleSidebar)) {
    e.preventDefault();
    explorerVisible.value = !explorerVisible.value;
    return;
  }
  if (e.altKey && e.key === "ArrowLeft") {
    e.preventDefault();
    scoutWidth.value = Math.max(260, scoutWidth.value - 40);
    return;
  }
  if (e.altKey && e.key === "ArrowRight") {
    e.preventDefault();
    scoutWidth.value = Math.min(2800, scoutWidth.value + 40);
    return;
  }
  if (e.altKey && e.key === "ArrowDown") {
    e.preventDefault();
    videoWidth.value = Math.max(160, videoWidth.value - 40);
    return;
  }
  if (e.altKey && e.key === "ArrowUp") {
    e.preventDefault();
    videoWidth.value = Math.min(2800, videoWidth.value + 40);
  }
}

function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  return (
    target.tagName === "INPUT" ||
    target.tagName === "TEXTAREA" ||
    target.isContentEditable
  );
}

function loadSettings() {
  const autoPlayStored = localStorage.getItem("autoPlayOnSeek");
  autoPlayOnSeek.value =
    autoPlayStored !== null ? autoPlayStored === "true" : true;
  const autoAdvanceStored = localStorage.getItem("autoAdvanceMontage");
  autoAdvanceMontage.value =
    autoAdvanceStored !== null ? autoAdvanceStored === "true" : true;
  const muteStored = localStorage.getItem("muteOnAutoplayStart");
  muteOnAutoplayStart.value =
    muteStored !== null ? muteStored === "true" : false;
  const seekStepStored = localStorage.getItem("videoSeekStepSeconds");
  const parsed = Number(seekStepStored);
  seekStepSeconds.value = Number.isFinite(parsed) && parsed > 0 ? parsed : 3;
}

function startDrag(target: "scout" | "video", e: PointerEvent) {
  const handle = e.currentTarget as HTMLElement;
  handle.setPointerCapture(e.pointerId);
  bringPanelToFront(target);
  const pos = target === "scout" ? scoutPos.value : videoPos.value;
  dragState = {
    target,
    pointerId: e.pointerId,
    startX: e.clientX,
    startY: e.clientY,
    originX: pos.x,
    originY: pos.y,
  };
}

function startFilterDrag(e: PointerEvent) {
  if ((e.target as HTMLElement).closest("button, input")) return;
  (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  bringPanelToFront("filter");
  dragState = {
    target: "filter",
    pointerId: e.pointerId,
    startX: e.clientX,
    startY: e.clientY,
    originX: filterPos.value.x,
    originY: filterPos.value.y,
  };
}

function maybeStartEdgeResize(target: "scout" | "video", e: PointerEvent) {
  if (!(e.currentTarget instanceof HTMLElement)) return;
  if ((e.target as HTMLElement)?.closest(".drag-handle")) return;

  const rect = e.currentTarget.getBoundingClientRect();
  const nearLeft = e.clientX - rect.left <= EDGE_RESIZE_ZONE;
  const nearRight = rect.right - e.clientX <= EDGE_RESIZE_ZONE;
  const nearTop = e.clientY - rect.top <= EDGE_RESIZE_ZONE;
  const nearBottom = rect.bottom - e.clientY <= EDGE_RESIZE_ZONE;
  if (!nearLeft && !nearRight && !nearTop && !nearBottom) return;

  e.preventDefault();
  e.currentTarget.setPointerCapture(e.pointerId);
  bringPanelToFront(target);

  dragState = {
    target:
      nearRight && nearBottom
        ? target === "scout"
          ? "scout-resize"
          : "video-resize"
        : nearLeft && nearTop
          ? target === "scout"
            ? "scout-resize-top-left"
            : "video-resize-top-left"
          : nearRight && nearTop
            ? target === "scout"
              ? "scout-resize-top-right"
              : "video-resize-top-right"
            : nearLeft && nearBottom
              ? target === "scout"
                ? "scout-resize-bottom-left"
                : "video-resize-bottom-left"
              : nearTop
                ? target === "scout"
                  ? "scout-resize-top"
                  : "video-resize-top"
                : nearLeft
                  ? target === "scout"
                    ? "scout-resize-left"
                    : "video-resize-left"
                  : nearRight
                    ? target === "scout"
                      ? "scout-resize-right"
                      : "video-resize-right"
                    : target === "scout"
                      ? "scout-resize-bottom"
                      : "video-resize-bottom",
    pointerId: e.pointerId,
    startX: e.clientX,
    startY: e.clientY,
    originX: target === "scout" ? scoutPos.value.x : videoPos.value.x,
    originY: target === "scout" ? scoutPos.value.y : videoPos.value.y,
    originWidth: target === "scout" ? scoutWidth.value : videoWidth.value,
    originHeight: target === "scout" ? scoutHeight.value : videoHeight.value,
  };
}

function setResizeCursor(_target: "scout" | "video", e: PointerEvent) {
  onDrag(e);
  if (!(e.currentTarget instanceof HTMLElement)) return;
  const rect = e.currentTarget.getBoundingClientRect();
  const nearLeft = e.clientX - rect.left <= EDGE_RESIZE_ZONE;
  const nearRight = rect.right - e.clientX <= EDGE_RESIZE_ZONE;
  const nearTop = e.clientY - rect.top <= EDGE_RESIZE_ZONE;
  const nearBottom = rect.bottom - e.clientY <= EDGE_RESIZE_ZONE;
  e.currentTarget.style.cursor =
    nearRight && nearBottom
      ? "nwse-resize"
      : nearLeft && nearTop
        ? "nwse-resize"
        : nearRight && nearTop
          ? "nesw-resize"
          : nearLeft && nearBottom
            ? "nesw-resize"
            : nearTop
              ? "ns-resize"
              : nearLeft
                ? "ew-resize"
                : nearRight
                  ? "ew-resize"
                  : nearBottom
                    ? "ns-resize"
                    : "default";
}

function onDrag(e: PointerEvent) {
  if (!dragState || dragState.pointerId !== e.pointerId) return;
  const dx = e.clientX - dragState.startX;
  const dy = e.clientY - dragState.startY;

  if (
    dragState.target === "scout-resize" ||
    dragState.target === "scout-resize-right" ||
    dragState.target === "scout-resize-left" ||
    dragState.target === "scout-resize-top" ||
    dragState.target === "scout-resize-top-left" ||
    dragState.target === "scout-resize-top-right" ||
    dragState.target === "scout-resize-bottom-left" ||
    dragState.target === "scout-resize-bottom"
  ) {
    if (
      dragState.target === "scout-resize" ||
      dragState.target === "scout-resize-right" ||
      dragState.target === "scout-resize-left" ||
      dragState.target === "scout-resize-top-left" ||
      dragState.target === "scout-resize-bottom-left" ||
      dragState.target === "scout-resize-top-right"
    ) {
      const baseWidth = dragState.originWidth ?? scoutWidth.value;
      const nextWidth =
        dragState.target === "scout-resize-left" ||
        dragState.target === "scout-resize-top-left" ||
        dragState.target === "scout-resize-bottom-left"
          ? baseWidth - dx
          : baseWidth + dx;
      scoutWidth.value = Math.min(2800, Math.max(260, nextWidth));
      if (
        dragState.target === "scout-resize-left" ||
        dragState.target === "scout-resize-top-left" ||
        dragState.target === "scout-resize-bottom-left"
      ) {
        scoutPos.value = {
          ...scoutPos.value,
          x: Math.max(
            getMinPanelX(),
            (dragState.originX ?? scoutPos.value.x) + dx,
          ),
        };
      }
    }
    if (
      dragState.target === "scout-resize" ||
      dragState.target === "scout-resize-top" ||
      dragState.target === "scout-resize-top-left" ||
      dragState.target === "scout-resize-top-right" ||
      dragState.target === "scout-resize-bottom"
    ) {
      const baseHeight = dragState.originHeight ?? scoutHeight.value;
      const nextHeight =
        dragState.target === "scout-resize-top" ||
        dragState.target === "scout-resize-top-left" ||
        dragState.target === "scout-resize-top-right"
          ? baseHeight - dy
          : baseHeight + dy;
      scoutHeight.value = Math.min(2200, Math.max(360, nextHeight));
      if (
        dragState.target === "scout-resize-top" ||
        dragState.target === "scout-resize-top-left" ||
        dragState.target === "scout-resize-top-right"
      ) {
        scoutPos.value = {
          ...scoutPos.value,
          y: Math.max(0, (dragState.originY ?? scoutPos.value.y) + dy),
        };
      }
    }
    return;
  }

  if (
    dragState.target === "video-resize" ||
    dragState.target === "video-resize-right" ||
    dragState.target === "video-resize-left" ||
    dragState.target === "video-resize-top" ||
    dragState.target === "video-resize-top-left" ||
    dragState.target === "video-resize-top-right" ||
    dragState.target === "video-resize-bottom-left" ||
    dragState.target === "video-resize-bottom"
  ) {
    if (
      dragState.target === "video-resize" ||
      dragState.target === "video-resize-right" ||
      dragState.target === "video-resize-left" ||
      dragState.target === "video-resize-top-left" ||
      dragState.target === "video-resize-bottom-left" ||
      dragState.target === "video-resize-top-right"
    ) {
      const baseWidth = dragState.originWidth ?? videoWidth.value;
      const nextWidth =
        dragState.target === "video-resize-left" ||
        dragState.target === "video-resize-top-left" ||
        dragState.target === "video-resize-bottom-left"
          ? baseWidth - dx
          : baseWidth + dx;
      videoWidth.value = Math.min(2800, Math.max(160, nextWidth));
      if (
        dragState.target === "video-resize-left" ||
        dragState.target === "video-resize-top-left" ||
        dragState.target === "video-resize-bottom-left"
      ) {
        videoPos.value = {
          ...videoPos.value,
          x: Math.max(
            getMinPanelX(),
            (dragState.originX ?? videoPos.value.x) + dx,
          ),
        };
      }
    }
    if (
      dragState.target === "video-resize" ||
      dragState.target === "video-resize-top" ||
      dragState.target === "video-resize-top-left" ||
      dragState.target === "video-resize-top-right" ||
      dragState.target === "video-resize-bottom"
    ) {
      const baseHeight = dragState.originHeight ?? videoHeight.value;
      const nextHeight =
        dragState.target === "video-resize-top" ||
        dragState.target === "video-resize-top-left" ||
        dragState.target === "video-resize-top-right"
          ? baseHeight - dy
          : baseHeight + dy;
      videoHeight.value = Math.min(2200, Math.max(300, nextHeight));
      if (
        dragState.target === "video-resize-top" ||
        dragState.target === "video-resize-top-left" ||
        dragState.target === "video-resize-top-right"
      ) {
        videoPos.value = {
          ...videoPos.value,
          y: Math.max(0, (dragState.originY ?? videoPos.value.y) + dy),
        };
      }
    }
    return;
  }

  const next = {
    x: Math.max(getMinPanelX(), dragState.originX + dx),
    y: Math.max(0, dragState.originY + dy),
  };
  if (dragState.target === "filter") filterPos.value = next;
  else if (dragState.target === "scout") scoutPos.value = next;
  else videoPos.value = next;
}

function stopDrag(e: PointerEvent) {
  if (!dragState || dragState.pointerId !== e.pointerId) return;
  dragState = null;
}

onMounted(() => {
  window.addEventListener("keydown", onKeydown);
  loadSettings();
  window.addEventListener("storage", loadSettings);
  window.addEventListener("vbdb-settings-changed", loadSettings);
  listen<{ phase: string; current: number; total: number }>(
    "montage-export-progress",
    (event) => {
      if (!exportRunning.value) return;
      const { phase, current, total } = event.payload;
      if (phase === "clip") {
        exportStatus.value = `Exporting clip ${current}/${total - 1}...`;
      } else if (phase === "concat") {
        exportStatus.value = "Finalizing montage...";
      }
    },
  ).then((unlisten) => {
    unlistenMontageProgress = unlisten;
  });
  onBeforeUnmount(() => {
    window.removeEventListener("keydown", onKeydown);
    window.removeEventListener("storage", loadSettings);
    window.removeEventListener("vbdb-settings-changed", loadSettings);
    unlistenMontageProgress?.();
  });
});

function onAllImported() {
  explorerDrawer.value?.refresh();
}

function openMatch(payload: { id: number }) {
  selectedMatchIds.value = [payload.id];
}

function openMatches(payload: { ids: number[] }) {
  selectedMatchIds.value = [...payload.ids];
}

function onVideoSource(payload: { src: string; path: string; status: string }) {
  linkedVideoSrc.value = payload.src;
  videoStatus.value = payload.status;
}

function onSeekTime(payload: { time: number; clipEndTime: number | null }) {
  seekTime.value = payload.time;
  clipEndTime.value = payload.clipEndTime;
}

function onTogglePlayback() {
  playToggleToken.value += 1;
}

function parseOffset(raw: string, fallback: number): number {
  const trimmed = raw.trim();
  if (!trimmed) return fallback;
  const value = Number(trimmed);
  return Number.isFinite(value) ? value : fallback;
}

function formatOffset(value: number): string {
  if (value > 0) return `+${value}`;
  return String(value);
}

function focusClipOffset(target: "start" | "end") {
  const input = target === "start" ? clipStartInput.value : clipEndInput.value;
  input?.focus();
  input?.select();
}

function adjustClipOffset(target: "start" | "end", delta: number) {
  const current =
    target === "start" ? clipStartOffset.value : clipEndOffset.value;
  const next = parseOffset(current, target === "start" ? 0 : 8) + delta;
  if (target === "start") clipStartOffset.value = formatOffset(next);
  else clipEndOffset.value = formatOffset(next);
}

function onClipOffsetKeydown(target: "start" | "end", e: KeyboardEvent) {
  if (e.key !== "ArrowUp" && e.key !== "ArrowDown") return;
  e.preventDefault();
  adjustClipOffset(target, e.key === "ArrowUp" ? 1 : -1);
}

function onClipEnded() {
  if (!autoAdvanceMontage.value) return;
  scoutBox.value?.playNextMontageRow();
}

async function pickVideoFile() {
  const selected = await open({
    multiple: false,
    filters: [
      {
        name: "Video",
        extensions: ["mp4", "mov", "mkv", "avi", "wmv", "webm"],
      },
    ],
  });
  if (!selected || Array.isArray(selected)) return;
  manualVideoSrc.value = convertFileSrc(selected.replace(/\\/g, "/"));
}

function useLinkedVideo() {
  manualVideoSrc.value = "";
}

function addFilterRow(relation: RuleRelation) {
  codeFilters.value.push(createFilterRow(relation));
}

type FilterField =
  | "number"
  | "subType"
  | "combo"
  | "startZone"
  | "endZone"
  | "skillType"
  | "players";

const SKILL_FIELDS: Record<string, FilterField[]> = {
  S: ["number", "subType", "startZone", "endZone"],
  R: ["number", "subType", "startZone", "endZone", "skillType", "players"],
  A: [
    "number",
    "subType",
    "combo",
    "startZone",
    "endZone",
    "skillType",
    "players",
  ],
  B: ["number", "subType", "startZone", "endZone", "players"],
  D: ["number", "subType", "startZone", "endZone", "skillType", "players"],
  E: ["number", "subType", "startZone", "endZone"],
  F: ["number", "subType", "startZone", "endZone"],
};

function showField(filter: ChainFilterRow, field: FilterField): boolean {
  const skill = filter.skill.trim().toUpperCase();
  if (!skill) return true;
  return SKILL_FIELDS[skill]?.includes(field) ?? field !== "combo";
}

const baseSkillForDatalist = computed(() => {
  return codeFilters.value[0]?.skill?.trim().toUpperCase() ?? "";
});

function removeFilterRow(index: number) {
  if (index === 0) return;
  codeFilters.value.splice(index, 1);
}

function onSelectedClipsChange(
  clips: {
    row_id: number;
    match_id: number | null;
    match_name: string | null;
    video_path: string;
    start_time: number;
    end_time: number;
    code: string;
    video_time_seconds: number;
  }[],
) {
  selectedMontageClips.value = clips;
}

function onScoutEditStatusChange(payload: { status: string; dirtyCount: number }) {
  scoutEditStatus.value = payload.status;
  scoutDirtyEditCount.value = payload.dirtyCount;
}

async function exportSelectedMontage() {
  const clips =
    scoutBox.value?.getSelectedMontageClips() ?? selectedMontageClips.value;
  if (clips.length === 0 || exportRunning.value) return;
  const outputPath = await save({
    title: "Export montage",
    defaultPath: "montage-720p.mp4",
    filters: [{ name: "MP4", extensions: ["mp4"] }],
  });
  if (!outputPath) return;
  exportRunning.value = true;
  exportStatus.value = "Exporting montage...";
  try {
    const result = await exportMontageVideo({
      outputPath,
      clips,
    });
    exportStatus.value = `Exported ${result.clips_exported} clips to ${result.output_path}`;
  } catch (e) {
    exportStatus.value = e instanceof Error ? e.message : String(e);
  } finally {
    exportRunning.value = false;
  }
}
</script>

<template>
  <div class="seasons-layout">
    <ExplorerDrawer
      ref="explorerDrawer"
      v-model="explorerVisible"
      :selected-match-ids="selectedMatchIds"
      show-team-filter
      @open-match="openMatch"
      @open-matches="openMatches"
    />
    <div class="content-panel">
      <MatchEmptyState
        v-if="selectedMatchIds.length === 0"
        @all-imported="onAllImported"
      />
      <div v-else>
        <div
          class="main-filter-strip"
          :style="{
            transform: `translate(${filterPos.x}px, ${filterPos.y}px)`,
            zIndex: panelOrder.filter,
          }"
        >
          <div
            class="filter-drag-handle"
            @pointerdown="startFilterDrag($event)"
            @pointermove="onDrag"
            @pointerup="stopDrag"
            @pointercancel="stopDrag"
          >
            <button class="main-action-btn" @click="pickVideoFile">
              {{
                manualVideoSrc || linkedVideoSrc
                  ? "Change video"
                  : "Select video"
              }}
            </button>
            <button
              class="main-action-btn"
              :disabled="exportRunning || selectedMontageClips.length === 0"
              @click="exportSelectedMontage"
            >
              {{
                exportRunning
                  ? "Exporting..."
                  : `Export Montage (${selectedMontageClips.length})`
              }}
            </button>
            <div class="clip-window-list">
              <input
                ref="clipStartInput"
                v-model="clipStartOffset"
                class="main-code-search clip-offset"
                type="text"
                placeholder="start (s)"
                @keydown="onClipOffsetKeydown('start', $event)"
              />
              <input
                ref="clipEndInput"
                v-model="clipEndOffset"
                class="main-code-search clip-offset"
                type="text"
                placeholder="end (s)"
                @keydown="onClipOffsetKeydown('end', $event)"
              />
            </div>
            <span class="drag-dots">::</span>
          </div>

          <button
            v-if="manualVideoSrc"
            class="main-action-btn ghost"
            @click="useLinkedVideo"
          >
            Use linked
          </button>
          <div class="filter-stack">
            <div class="code-filter-list">
              <input
                v-model="codeFilters[0].team"
                class="main-code-search"
                type="text"
                list="dl-team"
                placeholder="* / a / team"
              />
              <input
                v-model="codeFilters[0].number"
                class="main-code-search"
                type="text"
                placeholder="number"
              />
              <input
                v-model="codeFilters[0].skill"
                class="main-code-search"
                type="text"
                list="dl-skill"
                placeholder="skill"
              />
              <input
                v-if="showField(codeFilters[0], 'subType')"
                v-model="codeFilters[0].subType"
                class="main-code-search"
                type="text"
                :list="baseSkillForDatalist ? 'dl-subtype' : undefined"
                placeholder="sub"
              />
              <input
                v-model="codeFilters[0].grade"
                class="main-code-search"
                type="text"
                list="dl-grade"
                placeholder="grade"
              />
              <input
                v-if="showField(codeFilters[0], 'combo')"
                v-model="codeFilters[0].combo"
                class="main-code-search"
                type="text"
                placeholder="combo"
              />
              <input
                v-if="showField(codeFilters[0], 'startZone')"
                v-model="codeFilters[0].startZone"
                class="main-code-search field-zone"
                type="text"
                list="dl-zone"
                placeholder="start"
              />
              <input
                v-if="showField(codeFilters[0], 'endZone')"
                v-model="codeFilters[0].endZone"
                class="main-code-search field-zone"
                type="text"
                list="dl-zone"
                placeholder="end"
              />
              <input
                v-if="showField(codeFilters[0], 'skillType')"
                v-model="codeFilters[0].skillType"
                class="main-code-search"
                type="text"
                :list="baseSkillForDatalist ? 'dl-skilltype' : undefined"
                placeholder="type"
              />
              <input
                v-if="showField(codeFilters[0], 'players')"
                v-model="codeFilters[0].players"
                class="main-code-search field-narrow"
                type="text"
                placeholder="plyr"
              />
              <button
                class="advanced-filter-btn"
                title="Advanced filters"
                @click="advancedFiltersOpen = true"
              >
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  height="20px"
                  viewBox="0 -960 960 960"
                  width="20px"
                  fill="#e3e3e3"
                >
                  <path
                    d="M216-192v-240h-72v-72h217v72h-73v240h-72Zm0-384v-192h72v192h-72Zm156 0v-72h72v-120h72v120h72v72H372Zm72 384v-312h72v312h-72Zm228 0v-120h-73v-72h217v72h-72v120h-72Zm0-264v-312h72v312h-72Z"
                  />
                </svg>
              </button>
            </div>
          </div>
        </div>
        <datalist id="dl-team">
          <option value="*"></option>
          <option value="a"></option>
        </datalist>
        <datalist id="dl-skill">
          <option v-for="s in SKILLS" :key="s.value" :value="s.value">
            {{ s.label }}
          </option>
        </datalist>
        <datalist id="dl-subtype">
          <option
            v-for="o in getSubTypes(baseSkillForDatalist)"
            :key="o.value"
            :value="o.value"
          >
            {{ o.label }}
          </option>
        </datalist>
        <datalist id="dl-grade">
          <option v-for="g in GRADES" :key="g.value" :value="g.value">
            {{ g.label }}
          </option>
        </datalist>
        <datalist id="dl-zone">
          <option v-for="z in ZONES" :key="z.value" :value="z.value"></option>
        </datalist>
        <datalist id="dl-skilltype">
          <option
            v-for="o in getSkillTypes(baseSkillForDatalist)"
            :key="o.value"
            :value="o.value"
          >
            {{ o.label }}
          </option>
        </datalist>
        <div
          v-if="advancedFiltersOpen"
          class="advanced-overlay"
          @click.self="advancedFiltersOpen = false"
        >
          <div class="advanced-dialog">
            <div class="advanced-header">
              <h3>Code filters</h3>
              <button
                class="advanced-close"
                @click="advancedFiltersOpen = false"
              >
                ×
              </button>
            </div>
            <div class="advanced-table-wrap">
              <table class="advanced-table">
                <thead>
                  <tr>
                    <th>Rule</th>
                    <th>Cond.</th>
                    <th>Team</th>
                    <th>No.</th>
                    <th>Skill</th>
                    <th>Sub</th>
                    <th>Grade</th>
                    <th>Combo</th>
                    <th>Start</th>
                    <th>End</th>
                    <th>Type</th>
                    <th>Players</th>
                    <th></th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="(filter, index) in codeFilters" :key="index">
                    <td>
                      <span v-if="index === 0" class="base-label">Base</span>
                      <select
                        v-else
                        v-model="filter.relation"
                        class="advanced-select"
                      >
                        <option
                          v-for="option in ruleOptions"
                          :key="option.value"
                          :value="option.value"
                        >
                          {{ option.label }}
                        </option>
                      </select>
                    </td>
                    <td>
                      <select
                        v-model="filter.condition"
                        class="advanced-input condition-select"
                      >
                        <option value="contains">In</option>
                        <option value="not_contains">Not in</option>
                      </select>
                    </td>
                    <td>
                      <input v-model="filter.team" class="advanced-input" />
                    </td>
                    <td>
                      <input v-model="filter.number" class="advanced-input" />
                    </td>
                    <td>
                      <input v-model="filter.skill" class="advanced-input" />
                    </td>
                    <td>
                      <input v-model="filter.subType" class="advanced-input" />
                    </td>
                    <td>
                      <input v-model="filter.grade" class="advanced-input" />
                    </td>
                    <td>
                      <input v-model="filter.combo" class="advanced-input" />
                    </td>
                    <td>
                      <input
                        v-model="filter.startZone"
                        class="advanced-input"
                      />
                    </td>
                    <td>
                      <input v-model="filter.endZone" class="advanced-input" />
                    </td>
                    <td>
                      <input
                        v-model="filter.skillType"
                        class="advanced-input"
                      />
                    </td>
                    <td>
                      <input v-model="filter.players" class="advanced-input" />
                    </td>
                    <td>
                      <button
                        v-if="index > 0"
                        class="advanced-remove"
                        @click="removeFilterRow(index)"
                      >
                        ×
                      </button>
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
            <div class="advanced-actions">
              <button
                class="main-action-btn ghost"
                @click="addFilterRow('equal')"
              >
                Add rule
              </button>
              <button
                class="main-action-btn"
                @click="advancedFiltersOpen = false"
              >
                Process
              </button>
            </div>
          </div>
        </div>
        <div
          class="floating-wrap scout-wrap"
          :style="{
            width: `${scoutWidth}px`,
            height: `${scoutHeight}px`,
            transform: `translate(${scoutPos.x}px, ${scoutPos.y}px)`,
            zIndex: panelOrder.scout,
          }"
          @pointerdown="maybeStartEdgeResize('scout', $event)"
          @pointermove="setResizeCursor('scout', $event)"
          @pointerup="stopDrag"
          @pointercancel="stopDrag"
        >
          <div
            class="drag-handle"
            :class="{ dirty: scoutDirtyEditCount > 0 }"
            @pointerdown="startDrag('scout', $event)"
            @pointermove="onDrag"
            @pointerup="stopDrag"
            @pointercancel="stopDrag"
          >
            <span class="drag-dots">::</span>
            <div v-if="scoutDirtyEditCount > 0" class="title-edit-status">
              <span>{{ scoutEditStatus }}</span>
              <button @pointerdown.stop @click.stop="scoutBox?.savePendingEdits()">
                Save edits
              </button>
              <button @pointerdown.stop @click.stop="scoutBox?.discardPendingEdits()">
                Discard
              </button>
            </div>
          </div>
          <ScoutLinesBox
            ref="scoutBox"
            :match-ids="selectedMatchIds"
            :code-filters="codeFilters"
            :clip-start-offset="parseOffset(clipStartOffset, 0)"
            :clip-end-offset="parseOffset(clipEndOffset, 8)"
            @video-source="onVideoSource"
            @seek-time="onSeekTime"
            @toggle-playback="onTogglePlayback"
            @selected-clips-change="onSelectedClipsChange"
            @edit-status-change="onScoutEditStatusChange"
          />
        </div>
        <div v-if="exportStatus" class="export-status">
          {{ exportStatus }}
        </div>
        <div v-if="videoStatus" class="video-status">
          {{ videoStatus }}
        </div>
        <div
          class="floating-wrap video-wrap"
          :style="{
            width: `${videoWidth}px`,
            height: `${videoHeight}px`,
            transform: `translate(${videoPos.x}px, ${videoPos.y}px)`,
            zIndex: panelOrder.video,
          }"
          @pointerdown="maybeStartEdgeResize('video', $event)"
          @pointermove="setResizeCursor('video', $event)"
          @pointerup="stopDrag"
          @pointercancel="stopDrag"
        >
          <div
            class="drag-handle"
            @pointerdown="startDrag('video', $event)"
            @pointermove="onDrag"
            @pointerup="stopDrag"
            @pointercancel="stopDrag"
          >
            <span class="drag-dots">::</span>
          </div>
          <MatchVideoPanel
            ref="videoPanel"
            :source="manualVideoSrc || linkedVideoSrc"
            :seek-time="seekTime"
            :clip-end-time="clipEndTime"
            :play-toggle-token="playToggleToken"
            :auto-play-on-seek="autoPlayOnSeek"
            :mute-on-autoplay-start="muteOnAutoplayStart"
            @clip-ended="onClipEnded"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.seasons-layout {
  display: flex;
  height: 100%;
}

.content-panel {
  position: relative;
  flex: 1;
  z-index: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background: radial-gradient(
    circle at 50% 20%,
    var(--accent-soft),
    transparent 28rem
  );
}

.main-filter-strip {
  position: absolute;
  top: 0;
  left: 0;
  z-index: 5;
  width: max-content;
  max-width: calc(100% - 24px);
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  border: 1px solid var(--border-soft);
  border-radius: 8px;
  background: color-mix(in srgb, var(--surface) 80%, var(--bg));
  padding: 6px;
}

.advanced-filter-btn {
  width: 36px;
  height: 33px;
  border: 1px solid var(--border-soft);
  border-radius: 7px;
  background: transparent;
  color: var(--text-muted);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  flex-shrink: 0;
  margin-left: 6px;
}

.advanced-filter-btn:hover {
  background: var(--surface-soft);
  color: var(--fg);
}

.advanced-overlay {
  position: absolute;
  inset: 0;
  z-index: 20;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 72px;
  background: rgba(0, 0, 0, 0.24);
}

.advanced-dialog {
  width: min(1120px, calc(100vw - 120px));
  display: flex;
  flex-direction: column;
  gap: 12px;
  border: 1px solid var(--border-soft);
  border-radius: 10px;
  background: color-mix(in srgb, var(--surface) 88%, var(--bg));
  box-shadow: var(--shadow-lg);
  padding: 12px;
}

.advanced-table-wrap {
  overflow-x: auto;
  border: 1px solid var(--border-soft);
  border-radius: 8px;
  background: color-mix(in srgb, var(--bg) 78%, transparent);
}

.advanced-table {
  width: 100%;
  border-collapse: collapse;
  min-width: 1100px;
}

.advanced-table th {
  color: var(--text-muted);
  font-size: 11px;
  font-weight: 700;
  text-align: left;
  padding: 7px 6px;
  border-bottom: 1px solid var(--border-soft);
  background: color-mix(in srgb, var(--surface) 52%, transparent);
}

.advanced-table td {
  padding: 5px 6px;
  border-bottom: 1px solid var(--border-soft);
}

.advanced-table tr:last-child td {
  border-bottom: 0;
}

.base-label {
  display: inline-flex;
  align-items: center;
  min-height: 28px;
  color: var(--accent);
  font-size: 12px;
  font-weight: 700;
}

.advanced-input,
.advanced-select {
  width: 100%;
  min-width: 62px;
  border: 1px solid var(--border-soft);
  border-radius: 5px;
  background: color-mix(in srgb, var(--surface) 70%, transparent);
  color: var(--fg);
  font-size: 12px;
  padding: 6px 7px;
}

.advanced-select {
  min-width: 150px;
}

.condition-select {
  min-width: 56px;
  font-size: 11px;
  padding: 4px 5px;
}

.advanced-remove {
  width: 26px;
  height: 26px;
  border: 1px solid transparent;
  border-radius: 5px;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 16px;
}

.advanced-remove:hover {
  color: #e81123;
  border-color: color-mix(in srgb, #e81123 35%, var(--border-soft));
  background: color-mix(in srgb, #e81123 8%, transparent);
}

.advanced-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.advanced-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.advanced-header h3 {
  margin: 0;
  color: var(--fg);
  font-size: 14px;
}

.advanced-close {
  width: 26px;
  height: 26px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 18px;
}

.advanced-close:hover {
  background: var(--surface-soft);
  color: var(--fg);
}

.filter-drag-handle {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  height: 35px;
  margin-bottom: 8px;
  cursor: move;
  user-select: none;
  color: var(--text-muted);
  font-size: 12px;
  flex-shrink: 0;
  border-radius: 4px;
  transition: background 140ms ease;
}

.filter-drag-handle .drag-dots {
  margin-left: auto;
  padding: 0 8px;
}

.filter-drag-handle:hover {
  background: var(--surface-soft);
  color: var(--fg);
}

.main-action-btn {
  border: 1px solid var(--accent-border);
  background: var(--accent-soft);
  color: var(--accent);
  border-radius: 6px;
  padding: 6px 10px;
  cursor: pointer;
  flex-shrink: 0;
}

.main-action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.main-action-btn.ghost {
  background: transparent;
  color: var(--text-muted);
  border-color: var(--border-soft);
}

.main-code-search {
  flex: 0 0 96px;
  width: 96px;
  border: 1px solid var(--border-soft);
  border-radius: 0;
  background: color-mix(in srgb, var(--surface) 70%, transparent);
  color: var(--fg);
  font-size: 12px;
  padding: 7px 9px;
}

.code-filter-list {
  display: flex;
  gap: 0;
  width: 100%;
  flex-wrap: wrap;
}

.code-filter-list .main-code-search + .main-code-search {
  margin-left: -1px;
}

.code-filter-list .main-code-search:first-child {
  border-top-left-radius: 7px;
  border-bottom-left-radius: 7px;
}

.code-filter-list .main-code-search:nth-last-child(2),
.code-filter-list .main-code-search:last-child {
  border-top-right-radius: 7px;
  border-bottom-right-radius: 7px;
}

.filter-stack {
  display: flex;
  flex-direction: column;
  gap: 6px;
  width: auto;
}

.filter-mode {
  width: 64px;
  border: 1px solid var(--border-soft);
  border-radius: 8px;
  font-size: 11px;
  color: var(--text-muted);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  text-transform: uppercase;
}

.filter-relation {
  flex: 0 0 142px;
  width: 142px;
  border: 1px solid var(--border-soft);
  border-radius: 8px;
  background: color-mix(in srgb, var(--surface) 70%, transparent);
  color: var(--fg);
  font-size: 12px;
  padding: 7px 10px;
}

.filter-relation:focus {
  outline: 1px solid var(--accent-border);
}

.filter-row-actions {
  display: flex;
  gap: 6px;
}

.filter-remove {
  padding: 7px 10px;
}

.clip-window-list {
  display: flex;
  gap: 6px;
}

.clip-offset {
  width: 60px;
  flex-basis: 60px;
}

.filter-keys {
  display: flex;
  gap: 6px;
}

.filter-key {
  width: 94px;
  height: 30px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--border-soft);
  border-radius: 6px;
  color: var(--text-muted);
  font-size: 11px;
  font-weight: 600;
  background: color-mix(in srgb, var(--bg) 70%, transparent);
}

.main-code-search:focus {
  outline: 1px solid var(--accent-border);
}

.field-zone {
  flex: 0 0 62px;
  width: 62px;
}

.field-narrow {
  flex: 0 0 62px;
  width: 62px;
}

.floating-wrap {
  position: absolute;
  top: 0;
  left: 0;
  min-height: 300px;
}

.drag-handle {
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 10px;
  border: 1px solid var(--border-soft);
  border-bottom: 0;
  border-top-left-radius: 8px;
  border-top-right-radius: 8px;
  background: color-mix(in srgb, var(--surface) 80%, transparent);
  color: var(--text-muted);
  font-size: 12px;
  cursor: move;
  user-select: none;
}

.drag-handle.dirty {
  border-color: color-mix(in srgb, var(--accent) 45%, var(--border-soft));
  background: color-mix(in srgb, var(--accent-soft) 42%, var(--surface));
  color: var(--fg);
}

.title-edit-status {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-left: 10px;
  overflow: hidden;
  white-space: nowrap;
  cursor: default;
}

.title-edit-status span {
  overflow: hidden;
  text-overflow: ellipsis;
}

.title-edit-status button {
  border: 1px solid var(--border-soft);
  background: color-mix(in srgb, var(--bg) 78%, transparent);
  color: var(--fg);
  border-radius: 6px;
  padding: 2px 8px;
  font-size: 11px;
  cursor: pointer;
}

.drag-dots {
  letter-spacing: 2px;
  font-weight: 700;
}

.scout-wrap {
  min-width: 260px;
}

.video-wrap {
  min-width: 160px;
}

.export-status {
  position: absolute;
  left: 14px;
  bottom: 14px;
  z-index: 6;
  border: 1px solid var(--border-soft);
  background: color-mix(in srgb, var(--surface) 84%, var(--bg));
  border-radius: 8px;
  padding: 6px 10px;
  font-size: 12px;
  color: var(--text-muted);
  max-width: 640px;
}

.video-status {
  position: absolute;
  right: 14px;
  bottom: 14px;
  z-index: 6;
  border: 1px solid var(--border-soft);
  background: color-mix(in srgb, var(--surface) 84%, var(--bg));
  border-radius: 8px;
  padding: 6px 10px;
  font-size: 12px;
  color: var(--text-muted);
  max-width: 520px;
}
</style>
