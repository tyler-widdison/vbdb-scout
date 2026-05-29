import { invoke } from "@tauri-apps/api/core"
import type { Match } from "../../types/database"

export function getMatches(seasonId: number): Promise<Match[]> {
  return invoke("get_matches", { seasonId })
}

export function createMatch(seasonId: number, name: string): Promise<Match> {
  return invoke("create_match", { seasonId, name })
}

export function renameMatch(id: number, name: string): Promise<void> {
  return invoke("rename_match", { id, name })
}

export function deleteMatch(id: number): Promise<void> {
  return invoke("delete_match", { id })
}
