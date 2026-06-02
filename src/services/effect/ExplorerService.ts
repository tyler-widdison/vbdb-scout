import { Context, Effect, Layer } from "effect"
import type { Association, Season } from "../../types/database"
import { TauriInvokeError } from "./tauri"
import * as api from "../api"

interface ExplorerState {
  associations: Association[]
  seasons: Season[]
  selectedAssociationId: number | null
  selectedSeasonIds: number[]
}

interface InitResult extends ExplorerState {
  initialized: boolean
}

interface ExplorerServiceShape {
  readonly init: (
    storedAssociationId: number | null,
    storedSeasonIds: number[],
  ) => Effect.Effect<InitResult, TauriInvokeError>

  readonly selectAssociation: (
    state: ExplorerState,
    id: number,
  ) => Effect.Effect<ExplorerState, TauriInvokeError>

  readonly refreshAfterImport: (
    state: ExplorerState,
  ) => Effect.Effect<ExplorerState, TauriInvokeError>

  readonly reloadAssociations: () => Effect.Effect<Association[], TauriInvokeError>
  readonly reloadSeasons: (
    associations: Association[],
  ) => Effect.Effect<Season[], TauriInvokeError>

  readonly addAssociation: (
    name: string,
  ) => Effect.Effect<Association[], TauriInvokeError>

  readonly addSeason: (
    associationId: number,
    name: string,
    currentSeasons: Season[],
  ) => Effect.Effect<Season[], TauriInvokeError>

  readonly renameAssociation: (
    id: number,
    name: string,
  ) => Effect.Effect<Association[], TauriInvokeError>

  readonly renameSeason: (
    id: number,
    name: string,
  ) => Effect.Effect<Season[], TauriInvokeError>

  readonly deleteAssociation: (
    id: number,
  ) => Effect.Effect<Association[], TauriInvokeError>

  readonly deleteSeason: (
    id: number,
  ) => Effect.Effect<Season[], TauriInvokeError>
}

function fromApi<A>(fn: () => Promise<A>): Effect.Effect<A, TauriInvokeError> {
  return Effect.async<A, TauriInvokeError>((resume) => {
    fn()
      .then((v) => resume(Effect.succeed(v)))
      .catch((e) => resume(Effect.fail(e)))
  })
}

const loadAllSeasons = (associations: Association[]): Effect.Effect<Season[], TauriInvokeError> =>
  Effect.gen(function* () {
    const chunks = yield* Effect.all(
      associations.map((a) =>
        fromApi(() => api.getSeasons(a.id)).pipe(
          Effect.map((rows) => rows.map((row) => ({ ...row, association_id: a.id }))),
        )
      ),
      { concurrency: "unbounded" },
    )
    return chunks.flat()
  })

const ensureDefaultAssociation = (
  associations: Association[],
): Effect.Effect<Association, TauriInvokeError> =>
  associations.length > 0
    ? Effect.succeed(associations[0])
    : fromApi(() => api.createAssociation("VBDB"))

const ensureUntitledSeason = (
  associationId: number,
  seasons: Season[],
): Effect.Effect<Season, TauriInvokeError> => {
  const existing = seasons.find((s) => s.association_id === associationId)
  if (existing) return Effect.succeed(existing)
  return fromApi(() => api.createSeason(associationId, "Untitled season"))
}

export function makeExplorerService(): ExplorerServiceShape {
  return {
    init(storedAssociationId, storedSeasonIds) {
      return Effect.gen(function* () {
        let associations = yield* fromApi(() => api.getAssociations())
        const defaultAssociation = yield* ensureDefaultAssociation(associations)
        associations = yield* fromApi(() => api.getAssociations())
        let seasons = yield* loadAllSeasons(associations)

        const hasStoredAssociation =
          storedAssociationId !== null &&
          associations.some((a) => a.id === storedAssociationId)

        const selectedAssociationId = hasStoredAssociation
          ? storedAssociationId
          : defaultAssociation.id

        const season = yield* ensureUntitledSeason(selectedAssociationId, seasons)
        seasons = [...seasons, ...(!seasons.includes(season) ? [season] : [])]

        const validSeasonIds = seasons
          .filter((s) => s.association_id === selectedAssociationId)
          .map((s) => s.id)

        const restored = storedSeasonIds.filter((id) => validSeasonIds.includes(id))
        const selectedSeasonIds = restored.length > 0 ? restored : [season.id]

        return {
          associations,
          seasons,
          selectedAssociationId,
          selectedSeasonIds,
          initialized: true,
        }
      })
    },

    selectAssociation(state, id) {
      return Effect.gen(function* () {
        const season = yield* ensureUntitledSeason(id, state.seasons)
        const seasons = !state.seasons.includes(season)
          ? [...state.seasons, season]
          : state.seasons

        const validSeasonIds = seasons
          .filter((s) => s.association_id === id)
          .map((s) => s.id)

        const preserved = state.selectedSeasonIds.filter((sid) => validSeasonIds.includes(sid))
        const selectedSeasonIds = preserved.length > 0 ? preserved : [season.id]

        return {
          ...state,
          seasons,
          selectedAssociationId: id,
          selectedSeasonIds,
        }
      })
    },

    refreshAfterImport(state) {
      return Effect.gen(function* () {
        const seasons = yield* loadAllSeasons(state.associations)
        if (!state.selectedAssociationId) return { ...state, seasons }

        const visibleIds = seasons
          .filter((s) => s.association_id === state.selectedAssociationId)
          .map((s) => s.id)

        return {
          ...state,
          seasons,
          selectedSeasonIds: visibleIds,
        }
      })
    },

    reloadAssociations() {
      return fromApi(() => api.getAssociations())
    },

    reloadSeasons(associations) {
      return loadAllSeasons(associations)
    },

    addAssociation(name) {
      return Effect.gen(function* () {
        yield* fromApi(() => api.createAssociation(name))
        return yield* fromApi(() => api.getAssociations())
      })
    },

    addSeason(associationId, name, currentSeasons) {
      return Effect.gen(function* () {
        const season = yield* fromApi(() => api.createSeason(associationId, name))
        return [...currentSeasons, season]
      })
    },

    renameAssociation(id, name) {
      return Effect.gen(function* () {
        yield* fromApi(() => api.renameAssociation(id, name))
        return yield* fromApi(() => api.getAssociations())
      })
    },

    renameSeason(id, name) {
      return Effect.gen(function* () {
        yield* fromApi(() => api.renameSeason(id, name))
        return yield* fromApi(() => api.getAssociations()).pipe(
          Effect.flatMap((associations) => loadAllSeasons(associations)),
        )
      })
    },

    deleteAssociation(id) {
      return Effect.gen(function* () {
        yield* fromApi(() => api.deleteAssociation(id))
        return yield* fromApi(() => api.getAssociations())
      })
    },

    deleteSeason(id) {
      return Effect.gen(function* () {
        yield* fromApi(() => api.deleteSeason(id))
        return yield* fromApi(() => api.getAssociations()).pipe(
          Effect.flatMap((associations) => loadAllSeasons(associations)),
        )
      })
    },
  }
}

export class ExplorerService extends Context.Tag("ExplorerService")<
  ExplorerService,
  ExplorerServiceShape
>() {
  static readonly Live = Layer.sync(ExplorerService, makeExplorerService)
}

export type { ExplorerState, InitResult, ExplorerServiceShape }
