import { Schema } from "effect"
import { tauri } from "../effect/tauri"

export function getTeamsForSeasons(seasonIds: number[]): Promise<string[]> {
  return tauri("get_teams_for_seasons", { seasonIds }, Schema.Array(Schema.String)) as Promise<string[]>
}
