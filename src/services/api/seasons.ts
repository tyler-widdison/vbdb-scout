import { Schema } from "effect"
import type { Season } from "../../types/database"
import { SeasonSchema } from "../effect/schemas"
import { tauri, tauriVoid } from "../effect/tauri"

export function getSeasons(associationId: number): Promise<Season[]> {
  return tauri("get_seasons", { associationId }, Schema.Array(SeasonSchema)) as Promise<Season[]>
}

export function createSeason(associationId: number, name: string): Promise<Season> {
  return tauri("create_season", { associationId, name }, SeasonSchema) as Promise<Season>
}

export function renameSeason(id: number, name: string): Promise<void> {
  return tauriVoid("rename_season", { id, name })
}

export function deleteSeason(id: number): Promise<void> {
  return tauriVoid("delete_season", { id })
}
