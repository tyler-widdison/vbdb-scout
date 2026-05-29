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

watch(selectedSeasonIds, async (ids) => {
  if (ids.length === 0) {
    teams.value = [];
    return;
  }
  teams.value = await api.getTeamsForSeasons(ids);
}, { immediate: true });

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
  setSelectedTeamNames([])
  await setSelectedAssociation(id)
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
      parts.push(`${result.failed.length} file(s) skipped (${result.failed[0].reason})`);
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
    <div class="home-wrap">
      <div class="home-top-actions">
        <div></div>
        <button class="upload-btn" :disabled="uploading" @click="uploadScoutFiles">
          {{ uploading ? "Importing..." : "Upload scout files" }}
        </button>
        <div></div>
      </div>
      <p v-if="uploadError" class="upload-error">{{ uploadError }}</p>
      <p v-if="uploadSuccess" class="upload-success">{{ uploadSuccess }}</p>
      <p class="load-note">
        Large file selections may take a while the first time while scout rows are cached. Future loads should be faster.
      </p>
      <div class="selector-card">
        <section class="col">
          <header class="head">
            <h2>Associations</h2>
            <div class="actions">
              <button
                @click="
                  openDialog(
                    'addAssociation',
                    'New association',
                    'Association name',
                  )
                "
              >
                +
              </button>
              <button
                :class="{ active: renameAssociationMode }"
                @click="renameAssociationMode = !renameAssociationMode"
              >
                &#x270E;
              </button>
            </div>
          </header>

          <button
            v-for="association in associations"
            :key="association.id"
            class="row row-btn"
            :class="{ selected: selectedAssociationId === association.id }"
            @click="onAssociationRowClick(association.id)"
          >
            <span class="row-left">
              <input
                type="radio"
                :checked="selectedAssociationId === association.id"
              />
              <span>{{ association.name }}</span>
            </span>
            <button
              v-if="renameAssociationMode"
              class="delete-btn"
              title="Delete association"
              @click.stop="removeAssociation(association.id)"
            >
              ×
            </button>
          </button>
        </section>

        <section class="col">
          <header class="head">
            <h2>Seasons</h2>
            <div class="actions">
              <button
                @click="openDialog('addSeason', 'New season', 'Season name')"
              >
                +
              </button>
              <button
                :class="{ active: renameSeasonMode }"
                @click="renameSeasonMode = !renameSeasonMode"
              >
                &#x270E;
              </button>
            </div>
          </header>

          <label class="row select-all">
            <input
              type="checkbox"
              :checked="allSeasonsSelected"
              @change="
                toggleAllSeasons(($event.target as HTMLInputElement).checked)
              "
            />
            <span>Select all seasons</span>
          </label>

          <label
            v-for="season in visibleSeasons"
            :key="season.id"
            class="row row-season"
            @click="onSeasonRowClick(season.id)"
          >
            <span class="row-left">
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
              <span>{{ season.name }}</span>
            </span>
            <button
              v-if="renameSeasonMode"
              class="delete-btn"
              title="Delete season"
              @click.stop="removeSeason(season.id)"
            >
              ×
            </button>
          </label>
        </section>

        <section class="col col-teams">
          <header class="head">
            <h2>Teams</h2>
            <div v-if="selectedTeamNames.length > 0" class="actions">
              <button @click="setSelectedTeamNames([])" title="Clear team filter">Clear</button>
            </div>
          </header>

          <div class="col-scroll">
            <input
              v-if="teams.length > 8"
              class="team-search"
              type="text"
              placeholder="Search teams..."
              v-model="teamSearch"
            />
            <div v-if="filteredTeams.length === 0" class="row disabled">
              <span>{{ teams.length === 0 ? 'No teams in selected seasons' : 'No matching teams' }}</span>
            </div>

            <label
              v-for="team in filteredTeams"
              :key="team"
              class="row"
            >
              <input
                type="checkbox"
                :checked="selectedTeamNames.includes(team)"
                @change="toggleTeamName(team, ($event.target as HTMLInputElement).checked)"
              />
              <span>{{ team }}</span>
            </label>
          </div>
        </section>
      </div>

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
}

.home-wrap {
  height: 100%;
  overflow: auto;
  padding: 16px;
}

.home-top-actions {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 420px));
  justify-content: center;
  gap: 10px;
}

.upload-btn {
  justify-self: center;
  border: 1px solid var(--accent-border);
  background: var(--accent-soft);
  color: var(--accent);
  border-radius: 8px;
  padding: 8px 12px;
  cursor: pointer;
}

.upload-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.upload-error {
  margin: 8px auto 0;
  max-width: 420px;
  color: #e81123;
  font-size: 12px;
}

.upload-success {
  margin: 8px auto 0;
  max-width: 420px;
  color: #4caf50;
  font-size: 12px;
}

.load-note {
  margin: 10px auto 0;
  max-width: 860px;
  border: 1px solid var(--border-soft);
  border-radius: 10px;
  background: color-mix(in srgb, var(--surface) 48%, transparent);
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.4;
  padding: 9px 12px;
}

.selector-card {
  display: grid;
  justify-content: center;
  grid-template-columns: repeat(3, minmax(0, 420px));
  gap: 10px;
  margin-top: 22px;
}

.viewer-card {
  border: 1px solid var(--border-soft);
  border-radius: 12px;
  background: color-mix(in srgb, var(--surface) 56%, transparent);
  overflow: hidden;
  min-height: 280px;
}

.tab-strip {
  display: flex;
  align-items: center;
  min-height: 38px;
  border-bottom: 1px solid var(--border-soft);
  padding: 0 8px;
  background: color-mix(in srgb, var(--surface) 42%, transparent);
}

.tab {
  border: 1px solid var(--border-soft);
  border-bottom-color: transparent;
  border-radius: 8px 8px 0 0;
  background: var(--bg);
  color: var(--fg);
  font-size: 12px;
  padding: 6px 10px;
}

.tab-placeholder {
  color: var(--text-muted);
  font-size: 12px;
}

.viewer-body {
  position: relative;
  min-height: 240px;
  padding: 8px;
}

.viewer-body.empty {
  padding: 0;
}

.col {
  border: 1px solid var(--border-soft);
  border-radius: 14px;
  background: color-mix(in srgb, var(--surface) 56%, transparent);
  padding: 12px;
  display: flex;
  flex-direction: column;
  max-height: calc(100vh - 140px);
}

.col-teams {
  max-height: 300px;
}

.col-scroll {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
}

.team-search {
  width: 100%;
  border: 1px solid var(--border-soft);
  border-radius: 6px;
  background: var(--bg);
  color: var(--fg);
  font-size: 12px;
  padding: 5px 8px;
  margin-bottom: 6px;
  outline: none;
}

.team-search::placeholder {
  color: var(--text-muted);
}

.team-search:focus {
  border-color: var(--accent-border);
}

.head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.head h2 {
  margin: 0;
  font-size: 14px;
}

.actions {
  display: flex;
  gap: 6px;
}

.actions button {
  background: var(--surface-soft);
  border: 1px solid var(--border-soft);
  color: var(--fg);
  width: 26px;
  height: 26px;
  border-radius: 6px;
  cursor: pointer;
}

.actions button.active {
  background: var(--accent-soft);
  border-color: var(--accent-border);
  color: var(--accent);
}

.row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--fg);
  padding: 6px 2px;
}

.row-btn {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  text-align: left;
  cursor: pointer;
}

.row-btn:hover {
  background: var(--surface-soft);
}

.row-btn.selected {
  background: var(--accent-soft);
  border-color: var(--accent-border);
}

.row-left {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.delete-btn {
  width: 22px;
  height: 22px;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
}

.delete-btn:hover {
  color: #e81123;
  border-color: color-mix(in srgb, #e81123 35%, var(--border-soft));
  background: color-mix(in srgb, #e81123 8%, transparent);
}

.row-season {
  justify-content: space-between;
}

.select-all {
  margin-bottom: 4px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border-soft);
}

.disabled {
  opacity: 0.6;
}
</style>
