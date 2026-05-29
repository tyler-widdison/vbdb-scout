import { computed, ref } from "vue"
import type { Association, Season } from "../types/database"
import * as api from "../services/api"

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

const selectedAssociation = computed(() =>
  associations.value.find((association) => association.id === selectedAssociationId.value) ?? null,
)

const selectedSeasons = computed(() =>
  seasons.value.filter((season) => selectedSeasonIds.value.includes(season.id)),
)

async function reloadAssociations() {
  associations.value = await api.getAssociations()
}

async function reloadSeasons() {
  const chunks = await Promise.all(
    associations.value.map(async (association) => ({
      associationId: association.id,
      rows: await api.getSeasons(association.id),
    })),
  )
  seasons.value = chunks.flatMap((chunk) => chunk.rows.map((row) => ({ ...row, association_id: chunk.associationId })))
}

async function ensureDefaultAssociation() {
  const existing = associations.value[0]
  if (existing) return existing
  return api.createAssociation("VBDB")
}

async function ensureUntitledSeason(associationId: number) {
  const existing = seasons.value.find((season) => season.association_id === associationId)
  if (existing) return existing
  const season = await api.createSeason(associationId, "Untitled season")
  seasons.value.push(season)
  return season
}

async function initExplorerContext() {
  if (initialized.value) return

  await reloadAssociations()
  const defaultAssociation = await ensureDefaultAssociation()
  await reloadAssociations()
  await reloadSeasons()

  const storedAssociationId = loadStoredAssociationId()
  const storedSeasonIds = loadStoredSeasonIds()
  const hasStoredAssociation =
    storedAssociationId !== null &&
    associations.value.some((association) => association.id === storedAssociationId)

  if (hasStoredAssociation) selectedAssociationId.value = storedAssociationId

  if (!selectedAssociationId.value) selectedAssociationId.value = defaultAssociation.id

  if (selectedAssociationId.value) {
    const season = await ensureUntitledSeason(selectedAssociationId.value)
    const validSeasonIds = seasons.value
      .filter((row) => row.association_id === selectedAssociationId.value)
      .map((row) => row.id)

    const restored = storedSeasonIds.filter((seasonId) => validSeasonIds.includes(seasonId))
    selectedSeasonIds.value = restored.length > 0 ? restored : [season.id]
  }

  persistSelection()

  initialized.value = true
}

async function setSelectedAssociation(id: number) {
  selectedAssociationId.value = id
  const season = await ensureUntitledSeason(id)

  const validSeasonIds = seasons.value
    .filter((row) => row.association_id === id)
    .map((season) => season.id)
  const preserved = selectedSeasonIds.value.filter((seasonId) => validSeasonIds.includes(seasonId))
  selectedSeasonIds.value = preserved.length > 0 ? preserved : [season.id]
  persistSelection()
}

function setSelectedSeasons(ids: number[]) {
  if (!selectedAssociationId.value) return

  const validSeasonIds = seasons.value
    .filter((season) => season.association_id === selectedAssociationId.value)
    .map((season) => season.id)
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
  await reloadSeasons()
  if (!selectedAssociationId.value) return
  const visibleIds = seasons.value
    .filter((season) => season.association_id === selectedAssociationId.value)
    .map((season) => season.id)
  selectedSeasonIds.value = visibleIds
  persistSelection()
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
