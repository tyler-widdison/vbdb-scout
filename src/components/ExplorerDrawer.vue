<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import SeasonTree from "./tree/SeasonTree.vue";

defineOptions({ inheritAttrs: false });

const props = withDefaults(
  defineProps<{
    modelValue?: boolean;
    showToggle?: boolean;
    showIcons?: boolean;
  }>(),
  {
    modelValue: true,
    showToggle: true,
    showIcons: true,
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
}>();

const treeRef = ref<InstanceType<typeof SeasonTree> | null>(null);
const listStyle = ref<"file" | "card">("file");
const STORAGE_WIDTH_KEY = "explorer:drawerWidth";
const MIN_WIDTH = 0;
const MAX_WIDTH = 600;
const DEFAULT_WIDTH = 260;
const COLLAPSE_THRESHOLD = 52;

const drawerWidth = ref(loadStoredWidth());
const resizeState = ref<{ startX: number; startWidth: number } | null>(null);

function loadStoredWidth(): number {
  const raw = localStorage.getItem(STORAGE_WIDTH_KEY);
  if (!raw) return DEFAULT_WIDTH;
  const val = Number(raw);
  return Number.isFinite(val) && val > 0 && val <= MAX_WIDTH
    ? val
    : DEFAULT_WIDTH;
}

const visible = computed({
  get: () => props.modelValue,
  set: (value: boolean) => emit("update:modelValue", value),
});

const drawerStyle = computed(() => {
  if (!visible.value) return undefined;
  return { width: `${drawerWidth.value}px` };
});

function refresh() {
  return treeRef.value?.refresh();
}

function loadListStyle() {
  listStyle.value =
    localStorage.getItem("explorerListStyle") === "card" ? "card" : "file";
}

function onResizeStart(e: PointerEvent) {
  (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  resizeState.value = { startX: e.clientX, startWidth: drawerWidth.value };
}

function onResizeMove(e: PointerEvent) {
  if (!resizeState.value) return;
  const dx = e.clientX - resizeState.value.startX;
  const next = resizeState.value.startWidth + dx;
  drawerWidth.value = Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, next));
}

function onResizeEnd() {
  if (!resizeState.value) return;
  resizeState.value = null;
  if (drawerWidth.value < COLLAPSE_THRESHOLD) {
    if (drawerWidth.value > 0) {
      drawerWidth.value = loadStoredWidth() || DEFAULT_WIDTH;
    }
    visible.value = false;
    return;
  }
  localStorage.setItem(STORAGE_WIDTH_KEY, String(drawerWidth.value));
}

function reopen() {
  const stored = loadStoredWidth();
  drawerWidth.value = stored;
  visible.value = true;
  localStorage.setItem(STORAGE_WIDTH_KEY, String(drawerWidth.value));
}

onMounted(() => {
  loadListStyle();
  window.addEventListener("storage", loadListStyle);
  window.addEventListener("vbdb-settings-changed", loadListStyle);
});

onBeforeUnmount(() => {
  window.removeEventListener("storage", loadListStyle);
  window.removeEventListener("vbdb-settings-changed", loadListStyle);
});

defineExpose({ refresh });
</script>

<template>
  <aside
    class="tree-drawer"
    :class="{
      collapsed: !visible,
      'file-list': listStyle === 'file',
      'no-icons': !showIcons,
    }"
    :style="drawerStyle"
  >
    <div class="explorer-shell" v-show="visible">
      <SeasonTree
        ref="treeRef"
        v-bind="$attrs"
        :show-header-actions="showIcons"
      />
    </div>
    <div
      v-if="visible"
      class="resize-handle"
      @pointerdown="onResizeStart"
      @pointermove="onResizeMove"
      @pointerup="onResizeEnd"
      @pointercancel="onResizeEnd"
    />
    <button
      v-if="visible && showToggle"
      class="explorer-toggle"
      @click="visible = false"
      title="Hide explorer"
    >
      Hide
    </button>
    <button
      v-if="!visible"
      class="explorer-reopen"
      @click="reopen"
      title="Show explorer"
    >
      &#x25B6;
    </button>
  </aside>
</template>

<style scoped>
.tree-drawer {
  position: relative;
  width: 260px;
  border-right: 1px solid var(--border-soft);
  background: color-mix(in srgb, var(--bg) 86%, var(--surface));
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
}

.tree-drawer.collapsed {
  width: auto;
  min-width: 0;
}

.resize-handle {
  position: absolute;
  top: 0;
  right: -3px;
  width: 6px;
  height: 100%;
  cursor: col-resize;
  z-index: 2;
}

.resize-handle:hover {
  background: var(--accent-border);
  opacity: 0.35;
}

.explorer-shell {
  flex: 1;
  min-height: 0;
}

.explorer-toggle {
  margin-top: auto;
  border: 0;
  border-top: 1px solid var(--border-soft);
  background: transparent;
  color: var(--text-muted);
  font-size: 11px;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  padding: 8px;
  cursor: pointer;
}

.explorer-toggle:hover {
  color: var(--fg);
  background: var(--surface-soft);
}

.explorer-reopen {
  border: 0;
  border-right: 1px solid var(--border-soft);
  background: color-mix(in srgb, var(--bg) 86%, var(--surface));
  color: var(--text-muted);
  font-size: 10px;
  padding: 12px 4px;
  cursor: pointer;
  writing-mode: vertical-rl;
  letter-spacing: 0.04em;
}

.explorer-reopen:hover {
  color: var(--fg);
  background: var(--surface-soft);
}

.file-list {
  background: color-mix(in srgb, var(--bg) 92%, #111);
}

.file-list :deep(.tree-header) {
  padding: 7px 10px;
  background: color-mix(in srgb, var(--surface) 34%, transparent);
}

.file-list :deep(.tree-title) {
  font-size: 11px;
  letter-spacing: 0.08em;
}

.file-list :deep(.tree-hint) {
  font-size: 10px;
}

.file-list :deep(.tree-body) {
  gap: 0;
  padding: 4px 0;
}

.file-list :deep(.file-row) {
  min-height: 26px;
  flex-direction: row;
  align-items: center;
  gap: 6px;
  border: 0;
  border-radius: 0;
  padding: 3px 28px 3px 24px;
  font-family:
    ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono",
    "Courier New", monospace;
}

.file-list :deep(.file-row::before) {
  content: "";
  position: absolute;
  left: 10px;
  width: 7px;
  height: 9px;
  border: 1px solid var(--text-muted);
  border-radius: 1px;
  opacity: 0.58;
}

.file-list :deep(.file-row:hover) {
  background: color-mix(in srgb, var(--surface-soft) 70%, transparent);
}

.file-list :deep(.file-row.active) {
  background: color-mix(in srgb, var(--accent-soft) 78%, transparent);
}

.file-list :deep(.name) {
  min-width: 0;
  padding-left: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 12px;
}

.file-list :deep(.video-badge) {
  margin-left: 6px;
}

.file-list :deep(.video-badge svg) {
  width: 13px;
  height: 13px;
}

.file-list :deep(.meta) {
  display: none;
}

.file-list :deep(.remove) {
  top: 3px;
  right: 5px;
}

.file-list.no-icons :deep(.file-row) {
  padding-left: 10px;
}

.file-list.no-icons :deep(.file-row::before),
.file-list.no-icons :deep(.video-badge) {
  display: none;
}
</style>
