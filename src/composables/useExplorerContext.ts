import { computed, ref } from "vue"
import type { Association, Season } from "../types/database"
import { Effect } from "effect"
import { makeExplorerService } from "../services/effect/ExplorerService"
import type { TauriInvokeError } from "../services/effect/tauri"

const svc = makeExplorerService()

const associations = ref<Association[]>([])
const seasons = ref<Season[]>([])
const selectedAssociationId = ref<number | null>(null)
const selectedSeasonIds = ref<number[]>([])
const selectedTeamNames = ref<string[]>([])
const initialized = ref(false)

const STORAGE_ASSOCIATION_KEY = "explorer:selectedAssociationId"
const STORAGE_SEASONS_KEY = "explorer:selectedSeasonIds"

function loadStoredAssociationId() {
  const raw = localStorage.getItem(STORAGE_ASSOCIATION_KEY)
  if (!raw) return null
  const id = Number(raw)
  return Number.isFinite(id) ? id : null
}

function loadStoredSeasonIds() {
  const raw = localStorage.getItem(STORAGE_SEASONS_KEY)
  if (!raw) return []
  try {
    const parsed = JSON.parse(raw)
    if (!Array.isArray(parsed)) return []
    return parsed
      .map((value) => Number(value))
      .filter((value) => Number.isFinite(value))
  } catch {
    return []
  }
}

function persistSelection() {
  if (selectedAssociationId.value === null) {
    localStorage.removeItem(STORAGE_ASSOCIATION_KEY)
  } else {
    localStorage.setItem(STORAGE_ASSOCIATION_KEY, String(selectedAssociationId.value))
  }
  localStorage.setItem(STORAGE_SEASONS_KEY, JSON.stringify(selectedSeasonIds.value))
}

function runPromise<A>(effect: Effect.Effect<A, TauriInvokeError>): Promise<A> {
  return Effect.runPromise(effect)
}

const selectedAssociation = computed(() =>
  associations.value.find((a) => a.id === selectedAssociationId.value) ?? null,
)

const selectedSeasons = computed(() =>
  seasons.value.filter((s) => selectedSeasonIds.value.includes(s.id)),
)

async function initExplorerContext() {
  if (initialized.value) return
  const result = await runPromise(
    svc.init(loadStoredAssociationId(), loadStoredSeasonIds()),
  )
  associations.value = result.associations
  seasons.value = result.seasons
  selectedAssociationId.value = result.selectedAssociationId
  selectedSeasonIds.value = result.selectedSeasonIds
  persistSelection()
  initialized.value = result.initialized
}

async function reloadAssociations() {
  associations.value = await runPromise(svc.reloadAssociations())
}

async function reloadSeasons() {
  seasons.value = await runPromise(svc.reloadSeasons(associations.value))
}

async function setSelectedAssociation(id: number) {
  const result = await runPromise(
    svc.selectAssociation(
      {
        associations: associations.value,
        seasons: seasons.value,
        selectedAssociationId: selectedAssociationId.value,
        selectedSeasonIds: selectedSeasonIds.value,
      },
      id,
    ),
  )
  seasons.value = result.seasons
  selectedAssociationId.value = result.selectedAssociationId
  selectedSeasonIds.value = result.selectedSeasonIds
  persistSelection()
}

function setSelectedSeasons(ids: number[]) {
  if (!selectedAssociationId.value) return
  const validSeasonIds = seasons.value
    .filter((s) => s.association_id === selectedAssociationId.value)
    .map((s) => s.id)
  selectedSeasonIds.value = ids.filter((id) => validSeasonIds.includes(id))
  persistSelection()
}

function setSelectedTeamNames(names: string[]) {
  selectedTeamNames.value = names
}

function toggleTeamName(name: string, checked: boolean) {
  if (checked) {
    if (!selectedTeamNames.value.includes(name)) {
      selectedTeamNames.value = [...selectedTeamNames.value, name]
    }
  } else {
    selectedTeamNames.value = selectedTeamNames.value.filter((n) => n !== name)
  }
}

async function refreshAfterImport() {
  const result = await runPromise(
    svc.refreshAfterImport({
      associations: associations.value,
      seasons: seasons.value,
      selectedAssociationId: selectedAssociationId.value,
      selectedSeasonIds: selectedSeasonIds.value,
    }),
  )
  seasons.value = result.seasons
  selectedSeasonIds.value = result.selectedSeasonIds
  persistSelection()
}

async function ensureUntitledSeason(associationId: number) {
  const existing = seasons.value.find((s) => s.association_id === associationId)
  if (existing) return existing
  const { createSeason } = await import("../services/api")
  const season = await createSeason(associationId, "Untitled season")
  seasons.value.push(season)
  return season
}

export function useExplorerContext() {
  return {
    associations,
    seasons,
    selectedAssociationId,
    selectedSeasonIds,
    selectedTeamNames,
    selectedAssociation,
    selectedSeasons,
    initExplorerContext,
    reloadAssociations,
    reloadSeasons,
    refreshAfterImport,
    ensureUntitledSeason,
    setSelectedAssociation,
    setSelectedSeasons,
    setSelectedTeamNames,
    toggleTeamName,
  }
}
