import { invoke } from "@tauri-apps/api/core"
import type { Season } from "../../types/database"

export function getSeasons(associationId: number): Promise<Season[]> {
  return invoke("get_seasons", { associationId })
}

export function createSeason(associationId: number, name: string): Promise<Season> {
  return invoke("create_season", { associationId, name })
}

export function renameSeason(id: number, name: string): Promise<void> {
  return invoke("rename_season", { id, name })
}

export function deleteSeason(id: number): Promise<void> {
  return invoke("delete_season", { id })
}
