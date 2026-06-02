<script setup lang="ts">
import { ask, open } from "@tauri-apps/plugin-dialog";
import { computed, onMounted, onBeforeUnmount, ref, watch } from "vue";
import { useRouter } from "vue-router";
import ExplorerDrawer from "../components/ExplorerDrawer.vue";
import NamePromptDialog from "../components/common/NamePromptDialog.vue";
import { useExplorerContext } from "../composables/useExplorerContext";
import * as api from "../services/api";

const {
  associations,
  seasons,
  selectedAssociationId,
  selectedSeasonIds,
  selectedTeamNames,
  initExplorerContext,
  reloadAssociations,
  reloadSeasons,
  setSelectedAssociation,
  setSelectedSeasons,
  setSelectedTeamNames,
  toggleTeamName,
  refreshAfterImport,
} = useExplorerContext();
const router = useRouter();

const renameAssociationMode = ref(false);
const renameSeasonMode = ref(false);
const uploading = ref(false);
const uploadError = ref("");
const uploadSuccess = ref("");

const dialogOpen = ref(false);
const dialogTitle = ref("");
const dialogLabel = ref("");
const dialogInitialValue = ref("");
const dialogAction = ref<
  "addAssociation" | "addSeason" | "renameAssociation" | "renameSeason"
>("addAssociation");
const dialogTargetId = ref<number | null>(null);

const visibleSeasons = computed(() =>
  seasons.value.filter(
    (season) => season.association_id === selectedAssociationId.value,
  ),
);

const allSeasonsSelected = computed(
  () =>
    visibleSeasons.value.length > 0 &&
    visibleSeasons.value.every((season) =>
      selectedSeasonIds.value.includes(season.id),
    ),
);

const teams = ref<string[]>([]);
const teamSearch = ref("");

const filteredTeams = computed(() => {
  if (!teamSearch.value) return teams.value;
  const q = teamSearch.value.toLowerCase();
  return teams.value.filter((team) => team.toLowerCase().includes(q));
});

const activeAssociation = computed(() =>
  associations.value.find((a) => a.id === selectedAssociationId.value),
);

const selectedCount = computed(() => selectedSeasonIds.value.length);

async function onScoutFilesImported() {
  await reloadSeasons();
  if (selectedSeasonIds.value.length > 0) {
    teams.value = await api.getTeamsForSeasons(selectedSeasonIds.value);
  }
}

onMounted(async () => {
  await initExplorerContext();
  window.addEventListener("scout-files-imported", onScoutFilesImported);
});

onBeforeUnmount(() => {
  window.removeEventListener("scout-files-imported", onScoutFilesImported);
});

watch(
  selectedSeasonIds,
  async (ids) => {
    if (ids.length === 0) {
      teams.value = [];
      return;
    }
    teams.value = await api.getTeamsForSeasons(ids);
  },
  { immediate: true },
);

function openDialog(
  action: typeof dialogAction.value,
  title: string,
  label: string,
  initialValue = "",
  targetId: number | null = null,
) {
  dialogAction.value = action;
  dialogTitle.value = title;
  dialogLabel.value = label;
  dialogInitialValue.value = initialValue;
  dialogTargetId.value = targetId;
  dialogOpen.value = true;
}

async function submitDialog(value: string) {
  const targetId = dialogTargetId.value;
  if (dialogAction.value === "addAssociation") {
    await api.createAssociation(value);
    await reloadAssociations();
  } else if (dialogAction.value === "addSeason") {
    if (!selectedAssociationId.value) return;
    await api.createSeason(selectedAssociationId.value, value);
    await reloadSeasons();
  } else if (dialogAction.value === "renameAssociation") {
    if (!targetId) return;
    await api.renameAssociation(targetId, value);
    await reloadAssociations();
  } else if (dialogAction.value === "renameSeason") {
    if (!targetId) return;
    await api.renameSeason(targetId, value);
    await reloadSeasons();
  }

  dialogOpen.value = false;
}

async function chooseAssociation(id: number) {
  setSelectedTeamNames([]);
  await setSelectedAssociation(id);
}

function toggleSeason(id: number, checked: boolean) {
  const next = checked
    ? [...selectedSeasonIds.value, id]
    : selectedSeasonIds.value.filter((value) => value !== id);
  setSelectedSeasons(next);
}

function toggleAllSeasons(checked: boolean) {
  setSelectedSeasons(
    checked ? visibleSeasons.value.map((season) => season.id) : [],
  );
}

function onAssociationRowClick(id: number) {
  if (renameAssociationMode.value) {
    const row = associations.value.find((association) => association.id === id);
    if (!row) return;
    openDialog(
      "renameAssociation",
      "Rename association",
      "Association name",
      row.name,
      row.id,
    );
    renameAssociationMode.value = false;
    return;
  }

  chooseAssociation(id);
}

function onSeasonRowClick(id: number) {
  if (!renameSeasonMode.value) return;
  const row = seasons.value.find((season) => season.id === id);
  if (!row) return;
  openDialog("renameSeason", "Rename season", "Season name", row.name, row.id);
  renameSeasonMode.value = false;
}

async function removeAssociation(id: number) {
  const association = associations.value.find((row) => row.id === id);
  if (!association) return;

  const childSeasons = seasons.value.filter(
    (season) => season.association_id === id,
  );
  let filesCount = 0;

  if (childSeasons.length > 0) {
    const chunks = await Promise.all(
      childSeasons.map((season) => api.getMatches(season.id)),
    );
    filesCount = chunks.reduce((count, rows) => count + rows.length, 0);
  }

  const message =
    childSeasons.length > 0 || filesCount > 0
      ? `This will remove all seasons and files from "${association.name}". Are you sure?`
      : `Delete association "${association.name}"?`;

  const confirmed = await ask(message, {
    title: "Confirm association delete",
    kind: childSeasons.length > 0 || filesCount > 0 ? "warning" : "info",
    okLabel: "Delete",
    cancelLabel: "Cancel",
  });
  if (!confirmed) return;

  await api.deleteAssociation(id);
  await reloadAssociations();
  await reloadSeasons();

  if (selectedAssociationId.value === id) {
    const fallback = associations.value[0];
    if (fallback) await setSelectedAssociation(fallback.id);
  }
}

async function removeSeason(id: number) {
  const season = seasons.value.find((row) => row.id === id);
  if (!season) return;

  const files = await api.getMatches(id);
  const message =
    files.length > 0
      ? `This will remove all files from season "${season.name}". Are you sure?`
      : `Delete season "${season.name}"?`;

  const confirmed = await ask(message, {
    title: "Confirm season delete",
    kind: files.length > 0 ? "warning" : "info",
    okLabel: "Delete",
    cancelLabel: "Cancel",
  });
  if (!confirmed) return;

  await api.deleteSeason(id);
  await reloadSeasons();

  if (selectedSeasonIds.value.includes(id)) {
    setSelectedSeasons(
      selectedSeasonIds.value.filter((seasonId) => seasonId !== id),
    );
  }
}

function openMatch(payload: { id: number; name: string }) {
  router.push({
    name: "seasons",
    query: {
      matchId: String(payload.id),
      matchName: payload.name,
    },
  });
}

async function uploadScoutFiles() {
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
  const autoSeason = await api.getAutoSeason();

  uploading.value = true;
  try {
    const result = await api.importScoutFiles(
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
</script>

<template>
  <div class="home-layout">
    <ExplorerDrawer
      :show-toggle="false"
      :show-icons="false"
      :allow-multi-select="false"
      @open-match="openMatch"
    />
    <div class="workspace">
      <header class="workspace-header">
        <div class="header-left">
          <div class="header-titles">
            <p class="page-sub">
              <template v-if="activeAssociation">{{
                activeAssociation.name
              }}</template>
              <template v-else>Select an association to begin</template>
              <span v-if="selectedCount > 0" class="count-badge"
                >{{ selectedCount }} season{{
                  selectedCount > 1 ? "s" : ""
                }}</span
              >
            </p>
          </div>
        </div>
        <button
          class="import-cta"
          :disabled="uploading"
          @click="uploadScoutFiles"
        >
          <svg
            width="16"
            height="16"
            viewBox="0 0 16 16"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M8 10V2m0 0L5 5m3-3 3 3" />
            <path d="M2 10v2a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2v-2" />
          </svg>
          <span>{{ uploading ? "Importing\u2026" : "Import files" }}</span>
        </button>
      </header>

      <div v-if="uploadError" class="toast toast-err">{{ uploadError }}</div>
      <div v-if="uploadSuccess" class="toast toast-ok">{{ uploadSuccess }}</div>

      <div class="columns">
        <section class="col">
          <div class="col-label">
            <span>Associations</span>
            <div class="col-btns">
              <button
                class="micro-btn"
                @click="
                  openDialog(
                    'addAssociation',
                    'New association',
                    'Association name',
                  )
                "
                title="Add"
              >
                +
              </button>
              <button
                class="micro-btn"
                :class="{ editing: renameAssociationMode }"
                @click="renameAssociationMode = !renameAssociationMode"
                title="Rename"
              >
                &#x270E;
              </button>
            </div>
          </div>
          <div class="col-list">
            <button
              v-for="association in associations"
              :key="association.id"
              class="list-item association-item"
              :class="{ picked: selectedAssociationId === association.id }"
              @click="onAssociationRowClick(association.id)"
            >
              <span class="association-name">{{ association.name }}</span>
              <span class="association-meta">
                {{
                  seasons.filter((s) => s.association_id === association.id)
                    .length
                }}
              </span>
              <button
                v-if="renameAssociationMode"
                class="del-btn"
                title="Delete"
                @click.stop="removeAssociation(association.id)"
              >
                &times;
              </button>
            </button>
            <div v-if="associations.length === 0" class="empty">
              No associations yet
            </div>
          </div>
        </section>

        <section class="col col-seasons">
          <div class="col-label">
            <span>Seasons</span>
            <div class="col-btns">
              <button
                class="micro-btn"
                :disabled="!selectedAssociationId"
                @click="openDialog('addSeason', 'New season', 'Season name')"
                title="Add"
              >
                +
              </button>
              <button
                class="micro-btn"
                :class="{ editing: renameSeasonMode }"
                @click="renameSeasonMode = !renameSeasonMode"
                title="Rename"
              >
                &#x270E;
              </button>
            </div>
          </div>
          <div class="col-list">
            <label
              v-if="visibleSeasons.length > 0"
              class="list-item select-all-item"
            >
              <input
                type="checkbox"
                :checked="allSeasonsSelected"
                @change="
                  toggleAllSeasons(($event.target as HTMLInputElement).checked)
                "
              />
              <span class="select-all-text">Select all</span>
            </label>
            <label
              v-for="season in visibleSeasons"
              :key="season.id"
              class="list-item season-item"
              :class="{ checked: selectedSeasonIds.includes(season.id) }"
              @click="onSeasonRowClick(season.id)"
            >
              <input
                type="checkbox"
                :checked="selectedSeasonIds.includes(season.id)"
                @change="
                  toggleSeason(
                    season.id,
                    ($event.target as HTMLInputElement).checked,
                  )
                "
              />
              <span class="season-name">{{ season.name }}</span>
              <button
                v-if="renameSeasonMode"
                class="del-btn"
                title="Delete"
                @click.stop="removeSeason(season.id)"
              >
                &times;
              </button>
            </label>
            <div
              v-if="visibleSeasons.length === 0 && selectedAssociationId"
              class="empty"
            >
              No seasons &mdash; add one or import files
            </div>
            <div v-if="!selectedAssociationId" class="empty">
              Pick an association first
            </div>
          </div>
        </section>

        <section class="col col-teams">
          <div class="col-label">
            <span>Teams</span>
            <button
              v-if="selectedTeamNames.length > 0"
              class="micro-btn"
              @click="setSelectedTeamNames([])"
              title="Clear"
            >
              Clear
            </button>
          </div>
          <div class="col-list">
            <input
              v-if="teams.length > 6"
              class="filter-input"
              type="text"
              placeholder="Search Teams"
              v-model="teamSearch"
            />
            <label
              v-for="team in filteredTeams"
              :key="team"
              class="list-item team-item"
              :class="{ checked: selectedTeamNames.includes(team) }"
            >
              <input
                type="checkbox"
                :checked="selectedTeamNames.includes(team)"
                @change="
                  toggleTeamName(
                    team,
                    ($event.target as HTMLInputElement).checked,
                  )
                "
              />
              <span>{{ team }}</span>
            </label>
            <div v-if="filteredTeams.length === 0" class="empty">
              {{
                teams.length === 0
                  ? "Select seasons to see teams"
                  : "No matches"
              }}
            </div>
          </div>
        </section>
      </div>
      <div class="bottom-border"></div>
      <NamePromptDialog
        :open="dialogOpen"
        :title="dialogTitle"
        :label="dialogLabel"
        :initial-value="dialogInitialValue"
        @close="dialogOpen = false"
        @submit="submitDialog"
      />
    </div>
  </div>
</template>

<style scoped>
.home-layout {
  display: flex;
  height: 100%;
  min-height: 0;
  overflow: hidden;
}

.workspace {
  flex: 1;
  height: 100%;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.workspace-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-soft);
  flex-shrink: 0;
}

.bottom-border {
  border-bottom: 1px solid var(--border-soft);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.logo-mark {
  width: 34px;
  height: 34px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  background: var(--accent-soft);
  border: 1px solid var(--accent-border);
  color: var(--accent);
  font-family: "Cascadia Mono", monospace;
  font-weight: 700;
  font-size: 13px;
  letter-spacing: -0.04em;
  flex-shrink: 0;
}

.header-titles {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.page-title {
  font-size: 16px;
  margin: 0;
  letter-spacing: -0.02em;
}

.page-sub {
  font-size: 12px;
  color: var(--text-muted);
  display: flex;
  align-items: center;
  gap: 6px;
}

.count-badge {
  display: inline-flex;
  align-items: center;
  background: var(--accent-soft);
  color: var(--accent);
  border-radius: 4px;
  padding: 0 5px;
  font-size: 10px;
  font-family: "Cascadia Mono", monospace;
  line-height: 18px;
  font-weight: 600;
}

.import-cta {
  display: flex;
  align-items: center;
  gap: 7px;
  border: 1px solid var(--accent-border);
  border-radius: 7px;
  background: var(--accent-soft);
  color: var(--accent);
  padding: 7px 14px;
  font-family: "Cascadia Mono", monospace;
  font-weight: 600;
  font-size: 12px;
  cursor: pointer;
  transition:
    background 120ms ease,
    transform 80ms ease;
}

.import-cta:hover {
  background: color-mix(in srgb, var(--accent) 24%, transparent);
}

.import-cta:active {
  transform: scale(0.97);
}

.import-cta:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
}

.toast {
  padding: 6px 20px;
  font-size: 12px;
  border-bottom: 1px solid var(--border-soft);
}

.toast-err {
  color: #e81123;
  background: color-mix(in srgb, #e81123 6%, transparent);
  border-bottom-color: color-mix(in srgb, #e81123 20%, var(--border-soft));
}

.toast-ok {
  color: #4caf50;
  background: color-mix(in srgb, #4caf50 6%, transparent);
  border-bottom-color: color-mix(in srgb, #4caf50 20%, var(--border-soft));
}

.columns {
  display: flex;
  flex: 1;
  min-height: 0;
  max-height: 400px;
  gap: 0;
  border-top: 1px solid var(--border-soft);
}

.col {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  border-right: 1px solid var(--border-soft);
}

.col:last-child {
  border-right: none;
}

.col-label {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 9px 14px;
  font-family: "Cascadia Mono", monospace;
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--text-muted);
  border-bottom: 1px solid var(--border-soft);
  background: color-mix(in srgb, var(--surface) 25%, transparent);
  flex-shrink: 0;
}

.col-btns {
  display: flex;
  gap: 3px;
}

.micro-btn {
  width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--border-soft);
  border-radius: 4px;
  background: transparent;
  color: var(--fg);
  font-size: 12px;
  cursor: pointer;
  padding: 0;
  opacity: 0.5;
  transition:
    opacity 100ms ease,
    background 100ms ease;
}

.micro-btn:hover {
  opacity: 1;
  background: var(--surface-soft);
}

.micro-btn:disabled {
  opacity: 0.25;
  cursor: not-allowed;
}

.micro-btn.editing {
  background: var(--accent-soft);
  border-color: var(--accent-border);
  color: var(--accent);
  opacity: 1;
}

.col-list {
  flex: 1;
  overflow-y: auto;
  padding: 4px 6px;
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.list-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  padding: 6px 8px;
  border-radius: 5px;
  cursor: pointer;
  color: var(--fg);
  transition: background 80ms ease;
}

.association-item {
  border: none;
  background: transparent;
  text-align: left;
  width: 100%;
  font: inherit;
  justify-content: flex-start;
}

.association-item:hover {
  background: var(--surface-soft);
}

.association-item.picked {
  background: var(--accent-soft);
}

.association-item.picked .association-name {
  color: var(--accent);
  font-weight: 600;
}

.association-name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.association-meta {
  font-family: "Cascadia Mono", monospace;
  font-size: 10px;
  color: var(--text-muted);
  background: color-mix(in srgb, var(--surface) 60%, transparent);
  border-radius: 3px;
  padding: 1px 5px;
  line-height: 16px;
}

.season-item.checked {
  background: color-mix(in srgb, var(--accent) 8%, transparent);
}

.season-name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.select-all-item {
  margin-bottom: 2px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border-soft);
  border-radius: 0;
  font-size: 12px;
  color: var(--text-muted);
}

.select-all-text {
  font-size: 12px;
}

.team-item.checked {
  background: color-mix(in srgb, var(--accent) 8%, transparent);
}

.del-btn {
  width: 20px;
  height: 20px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 15px;
  flex-shrink: 0;
  transition:
    color 80ms ease,
    background 80ms ease;
}

.del-btn:hover {
  color: #e81123;
  background: color-mix(in srgb, #e81123 10%, transparent);
}

.filter-input {
  width: 100%;
  border: 1px solid var(--border-soft);
  border-radius: 5px;
  background: var(--bg);
  color: var(--fg);
  font-size: 12px;
  padding: 5px 8px;
  margin-bottom: 4px;
  outline: none;
}

.filter-input::placeholder {
  color: var(--text-muted);
}

.filter-input:focus {
  border-color: var(--accent-border);
}

.empty {
  color: var(--text-muted);
  font-size: 12px;
  padding: 10px 4px;
  opacity: 0.65;
}
</style>
