<script setup lang="ts">
import { ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { importScoutFiles } from "../../services/api/scoutFiles";
import { getAutoSeason } from "../../services/api/settings";
import { useExplorerContext } from "../../composables/useExplorerContext";

const emit = defineEmits<{
  allImported: [];
}>();

const uploading = ref(false);
const error = ref("");
const successMsg = ref("");
const {
  seasons,
  associations,
  selectedAssociationId,
  selectedSeasonIds,
  refreshAfterImport,
} = useExplorerContext();

async function uploadMatches() {
  error.value = "";
  successMsg.value = "";
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
  if (targetSeasons.length === 0) {
    error.value = "Select at least one season on Home page first.";
    return;
  }

  const targetSeason = targetSeasons[0];
  const autoSeason = await getAutoSeason();

  uploading.value = true;
  try {
    const result = await importScoutFiles(
      paths,
      selectedAssociation?.name ?? "VBDB",
      targetSeason.name,
      autoSeason,
    );
    if (result.imported.length === 0 && result.failed.length > 0) {
      error.value = `Import failed: ${result.failed[0].reason}`;
      return;
    }
    const parts: string[] = [];
    if (result.imported.length > 0) {
      parts.push(`Imported ${result.imported.length} file(s)`);
    }
    if (result.failed.length > 0) {
      parts.push(`${result.failed.length} file(s) skipped (${result.failed[0].reason})`);
    }
    successMsg.value = parts.join(". ");
    refreshAfterImport();
    emit("allImported");
    window.dispatchEvent(new CustomEvent("scout-files-imported"));
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    uploading.value = false;
  }
}
</script>

<template>
  <div class="empty-state">
    <p class="eyebrow">No match selected</p>
    <h1>Select a match or upload a match</h1>
    <p class="description">
      Choose a match from Explorer, or import a DataVolley scout file to start.
    </p>
    <button class="upload-btn" :disabled="uploading" @click="uploadMatches">
      {{ uploading ? "Importing..." : "Upload matches" }}
    </button>
    <p v-if="successMsg" class="success">{{ successMsg }}</p>
    <p v-if="error" class="error">{{ error }}</p>
  </div>
</template>

<style scoped>
.empty-state {
  width: min(520px, calc(100% - 48px));
  padding: 28px;
  border: 1px solid var(--border-soft);
  border-radius: 18px;
  background: color-mix(in srgb, var(--surface) 42%, transparent);
  box-shadow: var(--shadow-sm);
  text-align: center;
}

.eyebrow {
  margin: 0 0 8px;
  color: var(--accent);
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

h1 {
  margin: 0 0 10px;
  color: var(--fg);
  font-size: 24px;
}

.description {
  margin: 0 auto 20px;
  max-width: 380px;
  color: var(--text-muted);
  font-size: 14px;
}

.upload-btn {
  border: 1px solid var(--accent-border);
  border-radius: 10px;
  background: var(--accent-soft);
  color: var(--accent);
  cursor: pointer;
  font-weight: 700;
  padding: 10px 14px;
}

.upload-btn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--accent) 24%, transparent);
}

.upload-btn:disabled {
  cursor: default;
  opacity: 0.6;
}

.error {
  margin: 14px 0 0;
  color: var(--red);
  font-size: 13px;
}

.success {
  margin: 14px 0 0;
  color: #4caf50;
  font-size: 13px;
}
</style>
