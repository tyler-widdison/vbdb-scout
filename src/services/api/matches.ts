import { Schema } from "effect"
import type { Match } from "../../types/database"
import { MatchSchema } from "../effect/schemas"
import { tauri, tauriVoid } from "../effect/tauri"

export function getMatches(seasonId: number): Promise<Match[]> {
  return tauri("get_matches", { seasonId }, Schema.Array(MatchSchema)) as Promise<Match[]>
}

export function createMatch(seasonId: number, name: string): Promise<Match> {
  return tauri("create_match", { seasonId, name }, MatchSchema) as Promise<Match>
}

export function renameMatch(id: number, name: string): Promise<void> {
  return tauriVoid("rename_match", { id, name })
}

export function deleteMatch(id: number): Promise<void> {
  return tauriVoid("delete_match", { id })
}
