<script setup lang="ts">
import { ref, computed, watch, inject } from "vue";
import type { NodeData } from "../../types/tree";
import { TREE_INJECTION, nodeKey } from "../../types/tree";
import * as db from "../../services/api";
import TreeInput from "./TreeInput.vue";
import { loadNodeChildren } from "./explorerData";

const props = defineProps<{
  node: NodeData;
  fileExt: string;
}>();

const tree = inject(TREE_INJECTION)!;

const key = computed(() => nodeKey(props.node));
const isSelected = computed(() => tree.selectedKey.value === key.value);
const isActive = computed(() => isSelected.value && tree.focused.value);

const renaming = ref(false);
const renameValue = ref("");

watch(
  () => tree.renamingKey.value,
  (val) => {
    if (val === key.value) {
      renameValue.value = props.node.name;
      renaming.value = true;
    }
  },
);

async function confirmRename(value: string) {
  renaming.value = false;
  tree.stopRename();
  const nextName = value.trim();
  if (nextName && nextName !== props.node.name) {
    if (props.node.type === "association")
      await db.renameAssociation(props.node.id, nextName);
    else if (props.node.type === "season")
      await db.renameSeason(props.node.id, nextName);
    else await db.renameMatch(props.node.id, nextName);
    props.node.name = nextName;
  }
  tree.focus();
}

function cancelRename() {
  renaming.value = false;
  tree.stopRename();
  tree.focus();
}

async function toggleExpand() {
  if (props.node.type === "match") return;
  props.node.expanded = !props.node.expanded;
  if (props.node.expanded && !props.node.loaded)
    await loadNodeChildren(props.node);
}

async function selectNode() {
  tree.select(key.value);
  if (props.node.type === "match") {
    await tree.open(key.value);
  } else if (!props.node.expanded) {
    props.node.expanded = true;
    if (!props.node.loaded) await loadNodeChildren(props.node);
  }
}

async function createChild() {
  let createdKey = "";
  if (props.node.type === "association") {
    const season = await db.createSeason(props.node.id, "Untitled season");
    createdKey = `season-${season.id}`;
  } else if (props.node.type === "season") {
    const match = await db.createMatch(props.node.id, "Untitled match");
    createdKey = `match-${match.id}`;
  }
  if (!props.node.expanded) props.node.expanded = true;
  await loadNodeChildren(props.node);
  if (createdKey) tree.rename(createdKey);
}

async function deleteNode() {
  await tree.delete(key.value);
}
</script>

<template>
  <div class="tree-node">
    <div
      class="node-row"
      :class="{ selected: isSelected, active: isActive }"
      @click="selectNode"
      @dblclick="toggleExpand"
    >
      <span class="toggle" @click.stop="toggleExpand">
        <template v-if="node.type !== 'match'">
          {{ node.expanded ? "▾" : "▸" }}
        </template>
        <template v-else>&nbsp;</template>
      </span>
      <span class="node-icon">
        {{
          node.type === "association"
            ? "📁"
            : node.type === "season"
              ? "📂"
              : "📄"
        }}
      </span>
      <TreeInput
        v-if="renaming"
        v-model="renameValue"
        @confirm="confirmRename"
        @cancel="cancelRename"
      />
      <span v-else class="node-label">
        {{ node.name }}{{ node.type === "match" ? `.${fileExt}` : "" }}
      </span>
      <span class="node-actions">
        <button
          v-if="node.type !== 'match'"
          class="action-btn"
          @click.stop="createChild"
          :title="node.type === 'association' ? 'New season' : 'New match'"
        >
          +
        </button>
        <button
          class="action-btn delete"
          @click.stop="deleteNode"
          title="Delete"
        >
          ×
        </button>
      </span>
    </div>
    <div v-if="node.expanded" class="node-children">
      <TreeNode
        v-for="child in node.children"
        :key="`${child.type}-${child.id}`"
        :node="child"
        :file-ext="fileExt"
      />
    </div>
  </div>
</template>

<style scoped>
.tree-node {
  user-select: none;
}

.node-row {
  display: flex;
  align-items: center;
  gap: 4px;
  min-height: 26px;
  padding: 3px 6px;
  border-radius: 6px;
  cursor: pointer;
}

.node-row:hover {
  background: var(--surface-soft);
}

.node-row.selected {
  background: color-mix(in srgb, var(--surface-soft) 70%, transparent);
}

.node-row.active {
  background: var(--accent-soft);
  color: var(--accent);
}

.toggle {
  width: 16px;
  text-align: center;
  font-size: 10px;
  color: var(--fg);
  opacity: 0.5;
}

.node-icon {
  color: var(--text-muted);
  font-size: 12px;
  flex-shrink: 0;
}

.node-label {
  flex: 1;
  font-size: 13px;
  color: var(--fg);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.node-actions {
  display: none;
  gap: 2px;
  margin-left: auto;
}

.node-row:hover .node-actions {
  display: flex;
}

.action-btn {
  background: none;
  border: none;
  color: var(--fg);
  opacity: 0.5;
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  cursor: pointer;
  border-radius: 5px;
}

.action-btn:hover {
  opacity: 1;
  background: var(--surface-soft);
}

.action-btn.delete:hover {
  color: #e81123;
}

.node-children {
  padding-left: 16px;
}
</style>
