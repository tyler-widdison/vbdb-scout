import { invoke } from "@tauri-apps/api/core"

export function getTeamsForSeasons(seasonIds: number[]): Promise<string[]> {
  return invoke("get_teams_for_seasons", { seasonIds })
}
